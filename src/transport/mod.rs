/// Builder to read SML messages byte-wise from a stream
/// ```
/// use hackdose_sml_parser::transport::SMLMessageBuilder;
/// let mut builder = SMLMessageBuilder::Empty;
/// builder.record(&[0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01]);
/// builder.record(&[0x63, 0x01, 0x02]);
/// builder.record(&[0x1b, 0x1b, 0x1b, 0x1b, 0x1a, 0x01,0x02, 0x03]);
/// assert_eq!(builder, SMLMessageBuilder::Complete{ data: vec![0x63, 0x01, 0x02], rest: vec![]});
/// ```
#[derive(Eq, PartialEq, Debug)]
pub enum SMLMessageBuilder {
    Empty,
    IncompleteStartSignature(usize),
    Recording(Vec<u8>),
    Complete {
        /// the body of the message, omitting crc and header/footer
        data: Vec<u8>,
        /// the unprocessed rest of the byte stream
        rest: Vec<u8>,
    },
}

static START_SEQUENCE: &[u8] = &[0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01];
static END_SEQUENCE_WITHOUT_CRC: &[u8] = &[0x1b, 0x1b, 0x1b, 0x1b, 0x1a];

impl SMLMessageBuilder {
    pub fn record(&mut self, buf: &[u8]) {
        match self {
            SMLMessageBuilder::Empty | SMLMessageBuilder::IncompleteStartSignature(_) => {
                let start = match self {
                    SMLMessageBuilder::Empty => 0,
                    SMLMessageBuilder::IncompleteStartSignature(start) => *start,
                    SMLMessageBuilder::Recording(_) => todo!(),
                    SMLMessageBuilder::Complete { data: _, rest: _ } => todo!(),
                };
                let remainder_of_start_sequence = &START_SEQUENCE[start..];
                let remaining_start_sequence_bytes = remainder_of_start_sequence.len();

                let buffer_length = buf.len();

                struct MaximalOccurance {
                    index: usize,
                    length: usize,
                }
                let maximal_start_sequence_occurance = (0..buffer_length)
                    .map(|i| {
                        let window = &&buf[buffer_length - i - 1
                            ..usize::min(
                                buffer_length,
                                buffer_length - i - 1 + remainder_of_start_sequence.len(),
                            )];
                        let contained_length = contains(window, remainder_of_start_sequence);
                        if contained_length < remaining_start_sequence_bytes
                            && i + 1 > contained_length
                        {
                            MaximalOccurance {
                                index: buffer_length - i - 1,
                                length: 0,
                            }
                        } else {
                            MaximalOccurance {
                                index: buffer_length - i - 1,
                                length: contained_length,
                            }
                        }
                    })
                    .max_by_key(|item| item.length)
                    .unwrap_or(MaximalOccurance {
                        index: 0,
                        length: 0,
                    });

                if maximal_start_sequence_occurance.length == remaining_start_sequence_bytes {
                    *self = SMLMessageBuilder::Recording([].to_vec());
                    self.record(
                        &buf[maximal_start_sequence_occurance.index
                            + maximal_start_sequence_occurance.length..]
                            .to_vec(),
                    );
                } else if maximal_start_sequence_occurance.length > 0 {
                    *self = SMLMessageBuilder::IncompleteStartSignature(
                        maximal_start_sequence_occurance.length + start,
                    );
                } else if maximal_start_sequence_occurance.length == 0 && buf.len() > 0 {
                    *self = SMLMessageBuilder::Empty;
                };
            }

            SMLMessageBuilder::Recording(recorded) => {
                recorded.append(&mut buf.to_vec());
                let end = recorded
                    .windows(END_SEQUENCE_WITHOUT_CRC.len() + 3)
                    .enumerate()
                    .find(|(_, x)| x[..END_SEQUENCE_WITHOUT_CRC.len()] == *END_SEQUENCE_WITHOUT_CRC)
                    .map(|(index, _)| index);
                if let Some(end) = end {
                    let (message, rest) = recorded.split_at_mut(end);
                    *self = SMLMessageBuilder::Complete {
                        data: message.to_vec(),
                        rest: rest[END_SEQUENCE_WITHOUT_CRC.len() + 3..].to_vec(),
                    }
                }
            }
            _ => {}
        }
    }
}
fn contains(this: &[u8], that: &[u8]) -> usize {
    let mut counter = 0;
    for pair in this.iter().zip(that.iter()) {
        if pair.0 == pair.1 {
            counter += 1;
        } else {
            break;
        }
    }
    counter
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    pub fn extends_if_start_of_sequence_is_found() {
        let buf = vec![0x1b];
        let mut rec = SMLMessageBuilder::Empty;

        rec.record(&buf);
        assert_eq!(rec, SMLMessageBuilder::IncompleteStartSignature(1));
    }

    #[test]
    pub fn extends_if_start_of_sequence_is_found_anywhere() {
        let buf = vec![0xbb, 0x1b];
        let mut rec = SMLMessageBuilder::Empty;

        rec.record(&buf);
        assert_eq!(rec, SMLMessageBuilder::IncompleteStartSignature(1));
    }

    #[test]
    pub fn extends_more_if_bigger_portion_of_sequence_is_found() {
        let buf = vec![0x1b, 0x1b];
        let mut rec = SMLMessageBuilder::Empty;

        rec.record(&buf);
        assert_eq!(rec, SMLMessageBuilder::IncompleteStartSignature(2));
    }

    #[test]
    pub fn extends_more_if_even_bigger_portion_of_sequence_is_found() {
        let buf = vec![0x1b, 0x1b, 0x1b];
        let mut rec = SMLMessageBuilder::Empty;

        rec.record(&buf);
        assert_eq!(rec, SMLMessageBuilder::IncompleteStartSignature(3));
    }

    // incomplete occurences must be at the end

    #[test]
    pub fn incomplete_occurences_must_be_at_the_end() {
        let buf = vec![0x1b, 0x1b, 0x1b, 0x77];
        let mut rec = SMLMessageBuilder::Empty;

        rec.record(&buf);
        assert_eq!(rec, SMLMessageBuilder::Empty);
    }

    #[test]
    pub fn finds_complete_sequence() {
        let buf = &[0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01];

        let mut rec = SMLMessageBuilder::Empty;

        rec.record(buf);
        assert_eq!(rec, SMLMessageBuilder::Recording(vec![]));
    }

    #[test]
    pub fn extends_existing_start_sequence() {
        let buf = &[0x1b, 0x1b];

        let mut rec = SMLMessageBuilder::Empty;

        rec.record(buf);
        rec.record(buf);
        assert_eq!(rec, SMLMessageBuilder::IncompleteStartSignature(4));
    }

    #[test]
    pub fn returns_into_empty_if_start_signature_is_not_continued() {
        let buf = &[0x1b, 0x1b];

        let mut rec = SMLMessageBuilder::Empty;

        rec.record(buf);

        let buf = &[0x1b, 0x1a];

        rec.record(buf);
        assert_eq!(rec, SMLMessageBuilder::Empty);
    }

    #[test]
    pub fn leaves_unchanged_if_empty_buffer_is_recorded() {
        let buf = &[0x1b, 0x1b];

        let mut rec = SMLMessageBuilder::Empty;

        rec.record(buf);

        let buf = &[];

        rec.record(buf);
        assert_eq!(rec, SMLMessageBuilder::IncompleteStartSignature(2));
    }

    #[test]
    pub fn finds_complete_sequence_in_two_parts() {
        let buf = &[0x1b, 0x1b, 0x1b, 0x1b];
        let buf2 = &[0x01, 0x01, 0x01, 0x01];

        let mut rec = SMLMessageBuilder::Empty;

        rec.record(buf);
        rec.record(buf2);
        assert_eq!(rec, SMLMessageBuilder::Recording(vec![]));
    }

    #[test]
    pub fn puts_buffer_into_recorder() {
        let buf = &[0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01, 0x42, 0x43];

        let mut rec = SMLMessageBuilder::Empty;

        rec.record(buf);
        assert_eq!(rec, SMLMessageBuilder::Recording(vec![0x42, 0x43]));
    }

    #[test]
    pub fn extends_buffer_when_recording() {
        let buf = &[0x42, 0x43];

        let mut rec = SMLMessageBuilder::Recording(vec![]);

        rec.record(buf);
        assert_eq!(rec, SMLMessageBuilder::Recording(vec![0x42, 0x43]));
    }

    #[test]
    pub fn extends_recording_buffer() {
        let buf = &[0x44, 0x45];

        let mut rec = SMLMessageBuilder::Recording(vec![0x42, 0x43]);

        rec.record(buf);
        assert_eq!(
            rec,
            SMLMessageBuilder::Recording(vec![0x42, 0x43, 0x44, 0x45])
        );
    }

    #[test]
    pub fn puts_into_ended_state() {
        let buf = &[0x1b, 0x1b, 0x1b, 0x1b, 0x1a, 0x00, 0x01, 0x02, 0x03];

        let mut rec = SMLMessageBuilder::Recording(vec![0x42, 0x43]);

        rec.record(buf);
        assert_eq!(
            rec,
            SMLMessageBuilder::Complete {
                data: vec![0x42, 0x43],
                rest: vec![0x03]
            }
        );
    }

    #[test]
    pub fn keeps_rest() {
        let buf = &[0x1b, 0x1b, 0x1b, 0x1b, 0x1a, 0x00, 0x01, 0x02, 0x03];

        let mut rec = SMLMessageBuilder::Recording(vec![0x42, 0x43]);

        rec.record(buf);
        assert_eq!(
            rec,
            SMLMessageBuilder::Complete {
                data: vec![0x42, 0x43],
                rest: vec![0x03]
            }
        );
    }

    #[test]
    pub fn accepts_end_signature_in_two_parts() {
        let buf = &[0x1b, 0x1b, 0x1b, 0x1b];

        let mut rec = SMLMessageBuilder::Recording(vec![0x42, 0x43]);

        rec.record(buf);
        let buf = &[0x1a, 0x00, 0x01, 0x02, 0x03];
        rec.record(buf);

        assert_eq!(
            rec,
            SMLMessageBuilder::Complete {
                data: vec![0x42, 0x43],
                rest: vec![0x03]
            }
        );
    }

    #[test]
    pub fn perform_recording_and_finishing_in_one_step() {
        let buf = &[
            0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01, 0x42, 0x43, 0x1b, 0x1b, 0x1b, 0x1b,
            0x1a, 0x00, 0x01, 0x02,
        ];

        let mut rec = SMLMessageBuilder::Empty;

        rec.record(buf);

        assert_eq!(
            rec,
            SMLMessageBuilder::Complete {
                data: vec![0x42, 0x43],
                rest: vec![]
            }
        );
    }

    #[test]
    pub fn ignores_data_between_end_and_start() {
        let buf = &[
            0x7b, 0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01, 0x42, 0x43, 0x1b, 0x1b, 0x1b,
            0x1b, 0x1a, 0x00, 0x01, 0x02,
        ];

        let mut rec = SMLMessageBuilder::Empty;

        rec.record(buf);

        assert_eq!(
            rec,
            SMLMessageBuilder::Complete {
                data: vec![0x42, 0x43],
                rest: vec![]
            }
        );
    }

    #[test]
    pub fn takes_first_of_two_messages() {
        let buf = &[
            0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01, 0x42, 0x43, 0x1b, 0x1b, 0x1b, 0x1b,
            0x1a, 0x00, 0x01, 0x02, 0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01, 0x43, 0x1b,
            0x1b, 0x1b, 0x1b, 0x1a, 0x00, 0x02, 0x01,
        ];

        let mut rec = SMLMessageBuilder::Empty;

        rec.record(buf);

        assert_eq!(
            rec,
            SMLMessageBuilder::Complete {
                data: vec![0x42, 0x43],
                rest: vec![
                    0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01, 0x43, 0x1b, 0x1b, 0x1b, 0x1b,
                    0x1a, 0x00, 0x02, 0x01
                ]
            }
        );
    }
}
