use crate::controller::Command;
use crate::dur::SongOffsetSamples;
use crate::pattern::{NamedPattern, Pattern};
use crate::plugin_export::Context;
use crate::precise::{NoteType, PreciseEventType, PrecisePattern, SimpleNoteEvent};
use nih_plug::prelude::{nih_log, Params, ProcessStatus};
use rtrb::{Consumer, PopError};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::oneshot;

#[derive(Params)]
pub struct CodeParams {}

impl Default for CodeParams {
    fn default() -> Self {
        Self {}
    }
}

pub struct Code {
    // Plugin thread and command thread will communicate using these.
    pub commands_rx: Option<Consumer<Command>>,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub params: Arc<CodeParams>,
    pub playing: bool,

    patterns: HashMap<String, Pattern>,
    precise_patterns: HashMap<String, PrecisePattern>,
    future_events: HashMap<SongOffsetSamples, Vec<PreciseEventType>>,

    // Command thread will be shutdown by the plugin thread using this.
    tempo_prev_cycle: f64,
}

impl Default for Code {
    fn default() -> Self {
        Self {
            params: Arc::new(CodeParams::default()),
            playing: false,
            // We need the tempo and sample rate to properly initialize this.
            // Will be done on the first process() call.
            patterns: HashMap::new(),
            precise_patterns: HashMap::new(),
            future_events: HashMap::new(),
            commands_rx: None,
            shutdown_tx: None,
            tempo_prev_cycle: 0.0 as f64,
        }
    }
}

impl Code {
    pub fn cycle(
        &mut self,
        buf_size: usize,
        ctx: &Context,
    ) -> (ProcessStatus, Vec<PreciseEventType>) {
        let mut events: Vec<PreciseEventType> = Vec::new();

        if ctx.playing {
            if !self.playing {
                self.start(buf_size, ctx, &mut events);
            } else {
                self.play(buf_size, ctx, &mut events);
            }
        } else {
            if self.playing {
                self.stop(&mut events);
            }
        }
        self.tempo_prev_cycle = ctx.tempo;

        (ProcessStatus::Normal, events)
    }

    fn start(
        &mut self,
        buf_size: usize,
        ctx: &Context,
        events: &mut Vec<PreciseEventType>,
    ) -> ProcessStatus {
        self.playing = true;
        nih_log!(
            "starting CODE (sample rate {}) (tempo {})",
            ctx.sample_rate,
            ctx.tempo,
        );
        self.play(buf_size, ctx, events)
    }

    fn play(
        &mut self,
        buf_size: usize,
        ctx: &Context,
        events: &mut Vec<PreciseEventType>,
    ) -> ProcessStatus {
        if let Err(_) = self.process_commands(ctx, events) {
            return ProcessStatus::Error("error processing commands");
        }
        if ctx.tempo != self.tempo_prev_cycle {
            nih_log!("recomputing patterns after tempo change");
            self.recompute_patterns(ctx);
        }
        for event in self.get_events(ctx.pos_samples, buf_size) {
            if let PreciseEventType::Note(note) = event {
                match note.note_type {
                    NoteType::On => {
                        self.schedule_note_off(note, ctx.pos_samples);
                    }
                    _ => {}
                }
            }
            events.push(event);
        }
        for (song_pos_samples, fut_events) in self.get_future_events(ctx.pos_samples, buf_size) {
            for event in fut_events {
                events.push(event);
            }
            self.future_events.remove(&song_pos_samples);
        }
        ProcessStatus::Normal
    }

    fn stop(&mut self, events: &mut Vec<PreciseEventType>) -> ProcessStatus {
        self.playing = false;
        nih_log!("turning all notes off");
        self.turn_all_notes_off(events);
        ProcessStatus::Normal
    }

    fn schedule_note_off(&mut self, note: SimpleNoteEvent, song_pos_samples: i64) {
        let offset = (song_pos_samples as usize) + note.note_length_samples;
        if let Some(events) = self.future_events.get_mut(&offset) {
            events.push(PreciseEventType::Note(SimpleNoteEvent {
                note_type: NoteType::Off,
                timing: 0, // TBD when we actually send the note-off
                voice_id: note.voice_id,
                channel: note.channel,
                note: note.note,
                velocity: 0.0,
                note_length_samples: 0 as usize,
            }));
        } else {
            self.future_events.insert(
                offset,
                vec![PreciseEventType::Note(SimpleNoteEvent {
                    note_type: NoteType::Off,
                    timing: 0, // TBD when we actually send the note-off
                    voice_id: note.voice_id,
                    channel: note.channel,
                    note: note.note,
                    velocity: 0.0,
                    note_length_samples: 0 as usize,
                })],
            );
        }
    }

    fn get_future_events(
        &mut self,
        pos_samples: i64,
        buf_size: usize,
    ) -> Vec<(usize, Vec<PreciseEventType>)> {
        self.future_events
            .iter()
            .filter(|pair| {
                let (song_sample_position, _) = pair;
                **song_sample_position >= (pos_samples as usize)
                    && **song_sample_position > (pos_samples as usize) + buf_size
            })
            .map(|pair| {
                let (song_sample_position, events) = pair;
                (*song_sample_position, events.clone())
            })
            .collect()
    }

    fn get_events(&mut self, pos_samples: i64, buf_size: usize) -> Vec<PreciseEventType> {
        return self
            .precise_patterns
            .values_mut()
            .into_iter()
            .map(move |precise_pattern| {
                precise_pattern
                    .get_events(pos_samples as usize, (pos_samples as usize) + buf_size)
                    .into_iter()
            })
            .flatten()
            .collect();
    }

    fn recompute_patterns(&mut self, ctx: &Context) {
        for (name, pattern) in self.patterns.iter() {
            let mut playing = false;
            if let Some(existing_pattern) = self.precise_patterns.get(name) {
                playing = existing_pattern.playing;
            }
            nih_log!("recomputing pattern {}", name);
            let precise_pattern =
                PrecisePattern::from(&mut pattern.clone(), ctx.sample_rate, ctx.tempo, playing);
            self.precise_patterns
                .insert(name.clone(), precise_pattern.clone());
        }
    }

    fn process_commands(
        &mut self,
        ctx: &Context,
        events: &mut Vec<PreciseEventType>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(cmds) = self.commands_rx.as_mut() {
            match cmds.pop() {
                Ok(Command::PatternStart(pattern)) => self.start_pattern(ctx, pattern),
                Ok(Command::PatternStop(name)) => self.stop_pattern(&name, events),
                Ok(Command::PatternStopAll) => {
                    for (name, precp) in self.precise_patterns.iter_mut() {
                        nih_log!("stopping pattern {}", name);
                        precp.stop();
                    }
                    Ok(())
                }
                Ok(Command::PatternClear(name)) => {
                    self.precise_patterns.remove(&name);
                    Ok(())
                }
                Ok(Command::PatternClearAll) => {
                    self.precise_patterns.drain();
                    Ok(())
                }
                Err(PopError::Empty) => Ok(()),
            }
        } else {
            Ok(()) // Maybe panic here? This should be unreachable.
        }
    }

    fn start_pattern(
        &mut self,
        ctx: &Context,
        named_pattern: NamedPattern,
    ) -> Result<(), Box<dyn Error>> {
        let pattern_length = named_pattern.length_bars;
        nih_log!("starting pattern {}", named_pattern.name);
        let precise_pattern = PrecisePattern::from(
            &mut Pattern {
                channel: named_pattern.channel,
                length_bars: pattern_length,
                events: named_pattern.events.clone(),
            },
            ctx.sample_rate,
            ctx.tempo,
            true,
        );
        self.patterns.insert(
            named_pattern.name.clone(),
            Pattern {
                channel: named_pattern.channel,
                length_bars: pattern_length,
                events: named_pattern.events.clone(),
            },
        );
        self.precise_patterns
            .insert(named_pattern.name.clone(), precise_pattern.clone());
        nih_log!(
            "started pattern {} on channel {}",
            named_pattern.name,
            named_pattern.channel
        );
        Ok(())
    }

    fn stop_pattern(
        &mut self,
        name: &str,
        events: &mut Vec<PreciseEventType>,
    ) -> Result<(), Box<dyn Error>> {
        nih_log!("stopping pattern");
        match self.precise_patterns.get(name) {
            Some(precise_pattern) => {
                let mut clone = precise_pattern.clone();
                clone.playing = false;
                let prev_pattern = self.precise_patterns.insert(String::from(name), clone);
                if let Some(mut pattern) = prev_pattern {
                    let notes_playing = pattern.get_notes_playing();
                    for nev in notes_playing {
                        events.push(PreciseEventType::Note(nev));
                    }
                }
                nih_log!("stopped pattern {}", name);
                Ok(())
            }
            None => {
                nih_log!("no pattern with name {}", name);
                Ok(())
            }
        }
    }

    fn turn_all_notes_off(&mut self, events: &mut Vec<PreciseEventType>) -> () {
        for event in self
            .precise_patterns
            .values_mut()
            .into_iter()
            .map(move |ppat| ppat.get_notes_playing().into_iter())
            .flatten()
            .collect::<Vec<SimpleNoteEvent>>()
        {
            events.push(PreciseEventType::Note(event));
        }
    }
}
