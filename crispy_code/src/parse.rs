use crate::lex::Token;
use crate::pattern::Note;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    MissingAlternationAnchor,
    MissingAlternationDelimiter,
    MissingGroupDelimiter,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Alternation((Box<Element>, Vec<Element>)),
    Group(Vec<Element>),
    Note(Note),
    Rest,
    Tie,
}

#[allow(dead_code)]
pub struct ParsingStart;

pub struct ParsingPattern;

pub struct ParsingEnd {
    elems: Vec<Element>,
}

pub struct Parser<State> {
    state: State,
}

impl Parser<ParsingStart> {
    pub fn new() -> Parser<ParsingPattern> {
        Parser {
            state: ParsingPattern,
        }
    }

    fn new_group() -> Parser<ParsingGroup> {
        Parser {
            state: ParsingGroup,
        }
    }

    fn new_alternation() -> Parser<ParsingAlternation> {
        Parser {
            state: ParsingAlternation,
        }
    }
}

impl Parser<ParsingPattern> {
    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Parser<ParsingEnd>, ParseError> {
        let mut elems: Vec<Element> = vec![];

        if tokens.len() == 0 {
            return Ok(Parser {
                state: ParsingEnd { elems: elems },
            });
        }
        let mut idx = 0;

        while idx < tokens.len() {
            match tokens[idx] {
                Token::AlternationStart => {
                    let num_elems = elems.len();
                    if num_elems == 0 {
                        return Err(ParseError::MissingAlternationAnchor);
                    }
                    let alt = Parser::new_alternation();
                    let parsed = alt.parse((&tokens[(idx + 1)..]).to_vec())?;
                    let alt_elements = parsed.get_elements();
                    let tokens_consumed = parsed.get_tokens_consumed();
                    elems.push(Element::Alternation((
                        Box::new(elems[num_elems - 1].clone()),
                        alt_elements,
                    )));
                    idx += tokens_consumed + 1;
                    continue;
                }
                Token::AlternationEnd => {
                    // The GroupStart match arm should consume the `>`
                    return Err(ParseError::MissingAlternationDelimiter);
                }
                Token::GroupStart => {
                    let group = Parser::new_group();
                    let parsed = group.parse((&tokens[(idx + 1)..]).to_vec())?;
                    let group_elements = parsed.get_elements();
                    let tokens_consumed = parsed.get_tokens_consumed();
                    elems.push(Element::Group(group_elements));
                    idx += tokens_consumed + 1;
                    continue;
                }
                Token::GroupEnd => {
                    // The GroupStart match arm should consume the `]`
                    return Err(ParseError::MissingGroupDelimiter);
                }
                Token::NoteExpr(note) => {
                    elems.push(Element::Note(note));
                }
                Token::Rest => {
                    elems.push(Element::Rest);
                }
                Token::Tie => {
                    elems.push(Element::Tie);
                }
                Token::NoteRepeatGrouped(_) => {} // Safe to ignore
                Token::NoteRepeat(_) => {}        // Safe to ignore
                Token::NoteTie(_) => {}           // Safe to ignore
                Token::RestTie(_) => {}           // Safe to ignore
            }
            idx += 1
        }
        Ok(Parser {
            state: ParsingEnd { elems: elems },
        })
    }
}

impl Parser<ParsingEnd> {
    pub fn get_elements(&self) -> Vec<Element> {
        self.state.elems.clone()
    }
}

struct ParsingGroup;

struct GroupEnd {
    elems: Vec<Element>,
    tokens_consumed: usize,
}

impl Parser<ParsingGroup> {
    fn parse(&self, tokens: Vec<Token>) -> Result<Parser<GroupEnd>, ParseError> {
        let mut elems: Vec<Element> = vec![];

        if tokens.len() == 0 {
            // At the very least we need a `]`
            return Err(ParseError::MissingGroupDelimiter);
        }
        let mut idx = 0;

        while idx < tokens.len() {
            match tokens[idx] {
                Token::AlternationStart => {
                    let num_elems = elems.len();
                    if num_elems == 0 {
                        return Err(ParseError::MissingAlternationAnchor);
                    }
                    let alt = Parser::new_alternation();
                    let parsed = alt.parse((&tokens[(idx + 1)..]).to_vec())?;
                    let alt_elements = parsed.get_elements();
                    let tokens_consumed = parsed.get_tokens_consumed();
                    elems.push(Element::Alternation((
                        Box::new(elems[num_elems - 1].clone()),
                        alt_elements,
                    )));
                    idx += tokens_consumed + 1;
                    continue;
                }
                Token::AlternationEnd => {
                    // The GroupStart match arm should consume the `>`
                    return Err(ParseError::MissingAlternationDelimiter);
                }
                Token::GroupStart => {
                    let group = Parser::new_group();
                    let parsed = group.parse((&tokens[(idx + 1)..]).to_vec())?;
                    let group_elements = parsed.get_elements();
                    let tokens_consumed = parsed.get_tokens_consumed();
                    elems.push(Element::Group(group_elements));
                    idx += tokens_consumed + 1;
                    continue;
                }
                Token::GroupEnd => {
                    return Ok(Parser {
                        state: GroupEnd {
                            elems: elems,
                            tokens_consumed: idx + 1,
                        },
                    });
                }
                Token::NoteExpr(note) => {
                    elems.push(Element::Note(note));
                }
                Token::Rest => {
                    elems.push(Element::Rest);
                }
                Token::Tie => {
                    elems.push(Element::Tie);
                }
                Token::NoteRepeatGrouped(_) => {} // Safe to ignore
                Token::NoteRepeat(_) => {}        // Safe to ignore
                Token::NoteTie(_) => {}           // Safe to ignore
                Token::RestTie(_) => {}           // Safe to ignore
            }
            idx += 1
        }
        // If we made it here that means there was a missing `]`
        return Err(ParseError::MissingGroupDelimiter);
    }
}

impl Parser<GroupEnd> {
    fn get_elements(&self) -> Vec<Element> {
        self.state.elems.clone()
    }

    fn get_tokens_consumed(&self) -> usize {
        self.state.tokens_consumed
    }
}

struct ParsingAlternation;

struct AlternationEnd {
    elems: Vec<Element>,
    tokens_consumed: usize,
}

impl Parser<ParsingAlternation> {
    fn parse(&self, tokens: Vec<Token>) -> Result<Parser<AlternationEnd>, ParseError> {
        let mut elems: Vec<Element> = vec![];

        if tokens.len() == 0 {
            // At the very least we need a `>`
            return Err(ParseError::MissingAlternationDelimiter);
        }
        let mut idx = 0;

        while idx < tokens.len() {
            match tokens[idx] {
                Token::AlternationStart => {
                    let num_elems = elems.len();
                    if num_elems == 0 {
                        return Err(ParseError::MissingAlternationAnchor);
                    }
                    let alt = Parser::new_alternation();
                    let parsed = alt.parse((&tokens[(idx + 1)..]).to_vec())?;
                    let alt_elements = parsed.get_elements();
                    let tokens_consumed = parsed.get_tokens_consumed();
                    elems.push(Element::Alternation((
                        Box::new(elems[num_elems - 1].clone()),
                        alt_elements,
                    )));
                    idx += tokens_consumed + 1;
                    continue;
                }
                Token::AlternationEnd => {
                    return Ok(Parser {
                        state: AlternationEnd {
                            elems: elems,
                            tokens_consumed: idx + 1,
                        },
                    })
                }
                Token::GroupStart => {
                    let group = Parser::new_group();
                    let parsed = group.parse((&tokens[(idx + 1)..]).to_vec())?;
                    let group_elements = parsed.get_elements();
                    let tokens_consumed = parsed.get_tokens_consumed();
                    elems.push(Element::Group(group_elements));
                    idx += tokens_consumed + 1;
                    continue;
                }
                Token::GroupEnd => {
                    // The GroupStart match arm should consume the `]`
                    return Err(ParseError::MissingGroupDelimiter);
                }
                Token::NoteExpr(note) => {
                    elems.push(Element::Note(note));
                }
                Token::Rest => {
                    elems.push(Element::Rest);
                }
                Token::Tie => {
                    elems.push(Element::Tie);
                }
                Token::NoteRepeatGrouped(_) => {} // Safe to ignore
                Token::NoteRepeat(_) => {}        // Safe to ignore
                Token::NoteTie(_) => {}           // Safe to ignore
                Token::RestTie(_) => {}           // Safe to ignore
            }
            idx += 1
        }
        // If we made it here that means there was a missing `]`
        return Err(ParseError::MissingAlternationDelimiter);
    }
}

impl Parser<AlternationEnd> {
    fn get_elements(&self) -> Vec<Element> {
        self.state.elems.clone()
    }

    fn get_tokens_consumed(&self) -> usize {
        self.state.tokens_consumed
    }
}
