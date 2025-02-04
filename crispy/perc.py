import functools
import operator
from dataclasses import dataclass

from crispy.base_types import Event, Half, Note, PluginPattern, Sixteenth, Zero
from crispy.notes import NoteNumbers


@dataclass(slots=True)
class Perc:
    previous_event: Event | None = None

    def parse(self, definition: str) -> list[PluginPattern]:
        return [
            self.parse_line(line.strip())
            for line in definition.split("\n")
            if len(line.strip()) > 0
        ]

    def parse_line(self, line: str) -> PluginPattern:
        self.previous_event = None
        note_str, events_str = line.strip().split("=")
        note_num = NoteNumbers[note_str.strip()]
        if len(events_str.strip()) == 0:
            return PluginPattern(name=note_str.strip(), length_bars=Zero, events=[])
        events = [
            event
            for event in [
                self.parse_event(char, note_num)
                for char in events_str.strip()
                if not char.isspace()
            ]
            if event is not None
        ]
        return PluginPattern(
            name=note_str.strip(),
            events=events,
            length_bars=functools.reduce(operator.add, [ev.dur for ev in events]),
        )

    def parse_event(self, word: str, note_num: int) -> Event | None:
        if word == ".":
            event = Event(action="Rest", dur=Sixteenth)
        elif word == "X":
            event = Event(
                action=Note(
                    Note.Params(
                        note_num=note_num,
                        velocity=0.9,
                        dur=Half,
                    ),
                ),
                dur=Sixteenth,
            )
        elif word == "x":
            event = Event(
                action=Note(
                    Note.Params(
                        note_num=note_num,
                        velocity=0.4,
                        dur=Half,
                    ),
                ),
                dur=Sixteenth,
            )
        elif word == "+":
            assert self.previous_event is not None, "no event to tie"
            # lengthen the previous event
            self.previous_event.dur += Sixteenth
            return None
        else:
            raise ValueError(
                f"unsupported notation: {word} (line format is {self.usage()})"
            )
        self.previous_event = event
        return event

    def usage(cls) -> str:
        return "NOTE = [Xx_.]"


def perc(
    definition: str,
) -> list[PluginPattern]:
    """
    perc_pattern parses a DSL that is geared towards linear sequencing
    of individual notes.
    """
    _perc = Perc()

    try:
        return _perc.parse(definition)
    except ValueError as ex:
        print(f"could not parse pattern: {ex}")
        print(f"expected line format: {_perc.usage()}")
        raise ex
