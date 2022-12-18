use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

use crate::application::domain::{
    AnyValue, GetListResponseBody, GetOpenResponseBody, SmlListEntry, SmlMessageEnvelope,
    SmlMessages,
};

#[non_exhaustive]
#[derive(Debug)]
pub enum ParseError {
    Unknown,
}

pub type ParseResult<T> = Result<T, ParseError>;

/// Parse the body of an SML message (omitting header and footer)
pub fn parse_body(input: &[u8]) -> ParseResult<SmlMessages> {
    sml_parser::sml_body(input).map_err(|_| ParseError::Unknown)
}

/// Parse the whole SML message
pub fn parse_message(input: &[u8]) -> ParseResult<SmlMessages> {
    sml_parser::sml_messages(input).map_err(|_| ParseError::Unknown)
}

peg::parser! {
    grammar sml_parser<'a>() for [u8] {

        pub (crate) rule sml_body() -> SmlMessages
            = a:(sml_message_envelope())* { SmlMessages { messages: a } }

        pub (crate) rule sml_messages() -> SmlMessages
            = header() a:(sml_message_envelope())* footer() { SmlMessages { messages: a } }

        rule header() -> ()
            = ([0x1b] [0x1b] [0x1b] [0x1b] [0x01] [0x01] [0x01] [0x01])

        rule footer() -> ()
            = ([0x1b] [0x1b] [0x1b] [0x1b] [0x1a] [0..=255]*<3,3>)

        rule sml_message_envelope() -> SmlMessageEnvelope
            = [0x76] transaction_id() group_no() abort_on_error() a:sml_message_body() crc() end_of_message() { a }

        rule end_of_message() = [0x00]
        rule crc() = [0x63] any_number() any_number()

        rule sml_message_body() -> SmlMessageEnvelope
            = get_open_response() / get_list_response() / get_close_response() // and more types

        rule get_open_response() -> SmlMessageEnvelope
            = ([0x72] [0x63] [0x01] [0x01]) [0x76] a: get_open_response_content() { SmlMessageEnvelope::GetOpenResponse(a)}

        rule get_open_response_content() -> GetOpenResponseBody
            = [0x01] [0x01] req_file_id:string() server_id:string() [0x01] [0x01] { GetOpenResponseBody { server_id: server_id, req_file_id: req_file_id }}

        rule get_close_response() -> SmlMessageEnvelope
            = ([0x72] [0x63] [0x02] [0x01]) [0x71] get_close_response_content() { SmlMessageEnvelope::GetCloseResponse}

        rule get_close_response_content()
            = [0x01]

        rule get_list_response() -> SmlMessageEnvelope
            = ([0x72] [0x63] [0x07] [0x01]) [0x77] a: get_list_response_content() { SmlMessageEnvelope::GetListResponse(a)}

        rule list_signature()
            = [0x01]

        rule act_gateway_time()
            = ([0x01]*<0,1>)

        rule get_list_response_content() -> GetListResponseBody
            = [0x01] server_id:string() list_name:string() obscure_prefix_in_get_list_response() value_list:list_sml_value() list_signature() act_gateway_time() { GetListResponseBody { server_id: server_id, list_name: list_name, value_list: value_list }}

        rule obscure_prefix_in_get_list_response()
            = [0x72] [0x62] [0..=255] [0x65] [0..=255] [0..=255] [0..=255] [0..=255]

        rule list_sml_value1() -> Vec<SmlListEntry> = [0x71] n:(single_sml_value())*<1,1> { n }
        rule list_sml_value2() -> Vec<SmlListEntry> = [0x72] n:(single_sml_value())*<2,2> { n }
        rule list_sml_value3() -> Vec<SmlListEntry> = [0x73] n:(single_sml_value())*<3,3> { n }
        rule list_sml_value4() -> Vec<SmlListEntry> = [0x74] n:(single_sml_value())*<4,4> { n }
        rule list_sml_value5() -> Vec<SmlListEntry> = [0x75] n:(single_sml_value())*<5,5> { n }
        rule list_sml_value6() -> Vec<SmlListEntry> = [0x76] n:(single_sml_value())*<6,6> { n }
        rule list_sml_value7() -> Vec<SmlListEntry> = [0x77] n:(single_sml_value())*<7,7> { n }
        rule list_sml_value8() -> Vec<SmlListEntry> = [0x78] n:(single_sml_value())*<8,8> { n }
        rule list_sml_value9() -> Vec<SmlListEntry> = [0x79] n:(single_sml_value())*<9,9> { n }
        rule list_sml_value10() -> Vec<SmlListEntry> = [0x7a] n:(single_sml_value())*<10,10> { n }
        rule list_sml_value11() -> Vec<SmlListEntry> = [0x7b] n:(single_sml_value())*<11,11> { n }
        rule list_sml_value12() -> Vec<SmlListEntry> = [0x7c] n:(single_sml_value())*<12,12> { n }
        rule list_sml_value13() -> Vec<SmlListEntry> = [0x7d] n:(single_sml_value())*<13,13> { n }
        rule list_sml_value14() -> Vec<SmlListEntry> = [0x7e] n:(single_sml_value())*<14,14> { n }
        rule list_sml_value15() -> Vec<SmlListEntry> = [0x7f] n:(single_sml_value())*<15,15> { n }
        rule list_sml_value() -> Vec<SmlListEntry> = list_sml_value1()/list_sml_value1()/list_sml_value2()/list_sml_value3()/list_sml_value4()/list_sml_value5()/list_sml_value6()/list_sml_value7()/list_sml_value8()/list_sml_value9()/list_sml_value10()/list_sml_value11()/list_sml_value12()/list_sml_value13()/list_sml_value14()/list_sml_value15()
        rule single_sml_value() -> SmlListEntry
            = [0x77] obj_name: string() status: optional_unsigned_32() val_time: string() unit: (optional_unsigned_8()) scaler: scaler() value: value() sml_value_signature() { SmlListEntry { object_name: obj_name, status: status, value_time: val_time, unit, scaler: scaler, value: value }}

        rule scaler() -> Option<i8>
            = optional_signed_8()

        rule value() -> AnyValue
            = arbitrary()

        rule sml_value_signature()
            = [0x01]

        rule arbitrary() -> AnyValue =
            (v:string() { AnyValue::String(v)}) / (v:unsigned_16() { AnyValue::Unsigned(v as usize)}) / (v:signed_16() { AnyValue::Signed(v as isize)}) /
            (v:signed_64() { AnyValue::Signed(v as isize)}) / (v:signed_32() { AnyValue::Signed(v as isize)}) / (v:unsigned_32() { AnyValue::Unsigned(v as usize)})

        rule transaction_id()
            =(string4())

        rule group_no()
            =([0x62] any_number())

        rule abort_on_error()
            =([0x62] [0x00])

        rule message_checksum()
            = (any_number() any_number() any_number())

        rule any_number() -> u8
            = ([0..=255])

        rule unsigned_16() -> u16
            = [0x63] n:$([0..=255]*<2,2>) { {
                    let mut rdr = Cursor::new(n);
                    let res = rdr.read_u16::<BigEndian>().unwrap();
                    res
                }
            }

        rule unsigned_32() -> u32
            = [0x65] n:$([0..=255]*<4,4>) { {
                    let mut rdr = Cursor::new(n);
                    let res = rdr.read_u32::<BigEndian>().unwrap();
                    res
                }
            }

        rule signed_32() -> i32
            = [0x55] n:$([0..=255]*<4,4>) { {
                    let mut rdr = Cursor::new(n);
                    let res = rdr.read_i32::<BigEndian>().unwrap();
                    res
                }
            }

        rule unsigned_64() -> u64
            = [0x69] n:$([0..=255]*<8,8>) { {
                    let mut rdr = Cursor::new(n);
                    let res = rdr.read_u64::<BigEndian>().unwrap();
                    res
                }
            }


        rule signed_64() -> i64
            = [0x59] n:$([0..=255]*<8,8>) { {
                    let mut rdr = Cursor::new(n);
                    let res = rdr.read_i64::<BigEndian>().unwrap();
                    res
                }
            }

        pub rule signed_16() -> i16
            = [0x53] n:$([0..=255]*<2,2>) { {
                    let mut rdr = Cursor::new(n);
                    let res = rdr.read_i16::<BigEndian>().unwrap();
                    res
                }
            }

        rule signed_8() -> i8
            = [0x52] n:$([0..=255]*<1,1>) { {
                    let mut rdr = Cursor::new(n);
                    let res = rdr.read_i8().unwrap();
                    res
                }
            }

        rule unsigned_8() -> u8
            = [0x62] n:$([0..=255]*<1,1>) { {
                    let mut rdr = Cursor::new(n);
                    let res = rdr.read_u8().unwrap();
                    res
                }
            }

        rule optional_signed_16() -> Option<i16>
            = (v:signed_16() { Some(v) }) / ( [0x01] { None })

        rule optional_unsigned_16() -> Option<u16>
            = (v:unsigned_16() { Some(v) }) / ( [0x01] { None })

        rule optional_signed_8() -> Option<i8>
            = (v:signed_8() { Some(v) }) / ( [0x01] { None })

        rule optional_unsigned_8() -> Option<u8>
            = (v:unsigned_8() { Some(v) }) / ( [0x01] { None })

        rule optional_unsigned_32() -> Option<u32>
            = (v:unsigned_32() { Some(v) }) / ( [0x01] { None })

        rule string0() -> Vec<u8> = [0x01] n:(any_number())*<0,0> { n }
        rule string1() -> Vec<u8> = [0x02] n:(any_number())*<1,1> { n }
        rule string2() -> Vec<u8> = [0x03] n:(any_number())*<2,2> { n }
        rule string3() -> Vec<u8> = [0x04] n:(any_number())*<3,3> { n }
        rule string4() -> Vec<u8> = [0x05] n:(any_number())*<4,4> { n }
        rule string5() -> Vec<u8> = [0x06] n:(any_number())*<5,5> { n }
        rule string6() -> Vec<u8> = [0x07] n:(any_number())*<6,6> { n }
        rule string7() -> Vec<u8> = [0x08] n:(any_number())*<7,7> { n }
        rule string8() -> Vec<u8> = [0x09] n:(any_number())*<8,8> { n }
        rule string9() -> Vec<u8> = [0x0a] n:(any_number())*<9,9> { n }
        rule string10() -> Vec<u8> = [0x0b] n:(any_number())*<10,10> { n }
        rule string11() -> Vec<u8> = [0x0c] n:(any_number())*<11,11> { n }
        rule string12() -> Vec<u8> = [0x0d] n:(any_number())*<12,12> { n }
        rule string13() -> Vec<u8> = [0x0e] n:(any_number())*<13,13> { n }
        rule string14() -> Vec<u8> = [0x0f] n:(any_number())*<14,14> { n }
        rule string15() -> Vec<u8> = [0x10] n:(any_number())*<15,15> { n }
        rule string16() -> Vec<u8> = [0x11] n:(any_number())*<16,16> { n }
        rule string17() -> Vec<u8> = [0x81] [0x03] n:(any_number())*<17,17> { n }
        rule string18() -> Vec<u8> = [0x81] [0x04] n:(any_number())*<18,18> { n }
        rule string19() -> Vec<u8> = [0x81] [0x05] n:(any_number())*<19,19> { n }
        rule string20() -> Vec<u8> = [0x81] [0x06] n:(any_number())*<20,20> { n }
        rule string21() -> Vec<u8> = [0x81] [0x07] n:(any_number())*<21,21> { n }
        rule string22() -> Vec<u8> = [0x81] [0x08] n:(any_number())*<22,22> { n }
        rule string23() -> Vec<u8> = [0x81] [0x09] n:(any_number())*<23,23> { n }
        rule string24() -> Vec<u8> = [0x81] [0x0a] n:(any_number())*<24,24> { n }
        rule string25() -> Vec<u8> = [0x81] [0x0b] n:(any_number())*<25,25> { n }
        rule string26() -> Vec<u8> = [0x81] [0x0c] n:(any_number())*<26,26> { n }
        rule string27() -> Vec<u8> = [0x81] [0x0d] n:(any_number())*<27,27> { n }
        rule string28() -> Vec<u8> = [0x81] [0x0e] n:(any_number())*<28,28> { n }
        rule string29() -> Vec<u8> = [0x81] [0x0f] n:(any_number())*<29,29> { n }
        rule string30() -> Vec<u8> = [0x82] [0x00] n:(any_number())*<30,30> { n }
        rule string31() -> Vec<u8> = [0x82] [0x01] n:(any_number())*<31,31> { n }
        rule string32() -> Vec<u8> = [0x82] [0x02] n:(any_number())*<32,32> { n }
        rule string33() -> Vec<u8> = [0x82] [0x03] n:(any_number())*<33,33> { n }
        rule string34() -> Vec<u8> = [0x82] [0x04] n:(any_number())*<34,34> { n }
        rule string35() -> Vec<u8> = [0x82] [0x05] n:(any_number())*<35,35> { n }
        rule string36() -> Vec<u8> = [0x82] [0x06] n:(any_number())*<36,36> { n }
        rule string37() -> Vec<u8> = [0x82] [0x07] n:(any_number())*<37,37> { n }
        rule string38() -> Vec<u8> = [0x82] [0x08] n:(any_number())*<38,38> { n }
        rule string39() -> Vec<u8> = [0x82] [0x09] n:(any_number())*<39,39> { n }
        rule string40() -> Vec<u8> = [0x82] [0x0a] n:(any_number())*<40,40> { n }
        rule string41() -> Vec<u8> = [0x82] [0x0b] n:(any_number())*<41,41> { n }
        rule string42() -> Vec<u8> = [0x82] [0x0c] n:(any_number())*<42,42> { n }
        rule string43() -> Vec<u8> = [0x82] [0x0d] n:(any_number())*<43,43> { n }
        rule string44() -> Vec<u8> = [0x82] [0x0e] n:(any_number())*<44,44> { n }
        rule string45() -> Vec<u8> = [0x82] [0x0f] n:(any_number())*<45,45> { n }
        rule string46() -> Vec<u8> = [0x83] [0x00] n:(any_number())*<46,46> { n }
        rule string47() -> Vec<u8> = [0x83] [0x01] n:(any_number())*<47,47> { n }
        rule string48() -> Vec<u8> = [0x83] [0x02] n:(any_number())*<48,48> { n }
        rule string() -> Vec<u8> = string0()/string1()/string2()/string3()/string4()/string5()/string6()/string7()/string8()/string9()/string10()/string11()/string12()/string13()/string14()/string15()/string16()/string17()/string18()/string19()/string20()/string21()/string22()/string23()/string24()/string25()/string26()/string27()/string28()/string29()/string30()/string31()/string32()/string33()/string34()/string35()/string36()/string37()/string38()/string39()/string40()/string41()/string42()/string43()/string44()/string45()/string46()/string47()/string48()

    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn open() {
        //
        let example_open = vec![
            0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01, // header
            /* */ 0x76, // List with 6 entries
            /*      */ 0x05, 0x03, 0x2b, 0x18, 0x0f, // transactionId:
            /*      */ 0x62, 0x00, // groupNo:
            /*      */ 0x62, 0x00, //abortOnError:
            /*      */ 0x72, // messageBody: list with 2 entries
            /*          */ 0x63, 0x01, 0x01, // getOpenResponse:
            /*          */ 0x76, // list with 6 entries
            /*              */ 0x01, // codepage: no value
            /*              */ 0x01, // clientId: no value
            /*              */ 0x05, 0x04, 0x03, 0x02, 0x01, // reqFileId:
            /*              */ 0x0b, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
            0x0a, /*              */
            /*              */
            0x01, // refTime
            /*              */ 0x01, // smlVersion
            /*          */ 0x63, 0x49, 0x00, // CRC checksum of this message
            /*          */ 0x00, // end of this
            /* */ 0x1b, 0x1b, 0x1b, 0x1b, // Escape Sequenz
            /* */ 0x1a, 0x00, 0x70, 0xb2, // 1a + padding + CRC (2 bytes)
        ];

        let result = sml_parser::sml_messages(&example_open);

        assert_eq!(
            result,
            Ok(SmlMessages {
                messages: vec![SmlMessageEnvelope::GetOpenResponse(GetOpenResponseBody {
                    server_id: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a],
                    req_file_id: vec![0x04, 0x03, 0x02, 0x01]
                })]
            })
        )
    }

    #[test]
    pub fn get_list_response_body() {
        //
        let example_list = vec![
            /* */ 0x76, //
            /*      */ 0x05, 0x01, 0xD3, 0xD7, 0xBB, //
            /*      */ 0x62, 0x00, //
            /*      */ 0x62, 0x00, //
            /*      */ 0x72, //
            /*          */ 0x63, 0x07, 0x01, // getListResponse
            /*          */ 0x77, //
            /*              */ 0x01, // clientId / optional
            /*              */ 0x0B, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
            0x0a, // serverId
            /*              */ 0x07, 0x01, 0x00, 0x62, 0x0A, 0xFF, 0xFF, // listName
            /*              */ 0x72, // actSensorTime / optional
            /*                  */ 0x62, 0x01, // choice: secIndex
            /*                  */ 0x65, 0x01, 0x8A, 0x4D, 0x15, // secIndex (uptime)
            /*              */ 0x72, // valList
            /*                  */ 0x77, // SML_ListEntry
            /*                      */ 0x07, 0x81, 0x81, 0xC7, 0x82, 0x03,
            0xFF, // objName
            /*                      */ 0x01, // status
            /*                      */ 0x01, // valTime
            /*                      */ 0x01, // unit
            /*                      */ 0x01, // scaler
            /*                      */ 0x04, 0x49, 0x53,
            0x4B, // value -- Herstelleridentifikation (ISK)
            /*                      */ 0x01, // valueSignature / optional
            /*                  */ 0x77, // SML_ListEntry
            /*                      */ 0x07, 0x01, 0x00, 0x01, 0x08, 0x00,
            0xFF, // objName
            /*                      */ 0x65, 0x00, 0x00, 0x01, 0x82, // status / optional
            /*                      */ 0x01, // valTime / optional
            /*                      */ 0x62, 0x1E, // unit / optional
            /*                      */ 0x52, 0xFF, // scaler / optional
            /*                      */ 0x59, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, // Gesamtverbrauch
            /*                      */ 0x01, // valueSignature / optional
            /*                  */ 0x01, // listSignature / optional
            /*                  */ 0x01, // actGatewayTime / optional
            /*      */ 0x63, 0xC6, 0x12, // crc
            /*      */ 0x00, // end of message
        ];

        let result = sml_parser::sml_body(&example_list);

        assert_eq!(
            result,
            Ok(SmlMessages {
                messages: vec![SmlMessageEnvelope::GetListResponse(GetListResponseBody {
                    server_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                    list_name: vec![1, 0, 98, 10, 255, 255],
                    value_list: vec![
                        SmlListEntry {
                            object_name: vec![129, 129, 199, 130, 3, 255],
                            status: None,
                            value_time: vec![],
                            unit: None,
                            scaler: None,
                            value: AnyValue::String(vec![73, 83, 75])
                        },
                        SmlListEntry {
                            object_name: vec![1, 0, 1, 8, 0, 255],
                            status: Some(386),
                            value_time: vec![],
                            unit: Some(30),
                            scaler: Some(-1),
                            value: AnyValue::Signed(0)
                        }
                    ]
                })]
            })
        )
    }

    #[test]
    pub fn get_close_response() {
        let example_close = vec![
            0x1b, 0x1b, 0x1b, 0x1b, 0x01, 0x01, 0x01, 0x01, // header
            /*  */
            0x76, //
            /*      */ 0x05, 0x03, 0x2b, 0x18, 0x11, // transactionId:
            /*      */ 0x62, 0x00, // #groupNo:
            /*      */ 0x62, 0x00, // #abortOnError:
            /*      */ 0x72, //	messageBody:
            /*          */ 0x63, 0x02, 0x01, //	getCloseResponse:
            /*          */ 0x71, //
            /*              */ 0x01, // no value
            /*      */ 0x63, 0xfa, 0x36, // CRC
            /*      */ 0x00, //
            /* */ 0x1b, 0x1b, 0x1b, 0x1b, // escape sequence
            /* */ 0x1a, 0x00, 0x70, 0xb2, // 1a + padding + CRC (2 bytes)
        ];
        let result = sml_parser::sml_messages(&example_close);

        assert_eq!(
            result,
            Ok(SmlMessages {
                messages: vec![SmlMessageEnvelope::GetCloseResponse]
            })
        )
    }

    // From here on: Generate "generic types", should be solved by build scripts in the future
    #[test]
    pub fn generate_strings() {
        for i in 0..=14 {
            let length = format!("{:#04x}", i + 1);
            println!(
                "rule string{}() -> Vec<u8> = [{}] n:(any_number())*<{},{}> {{ n }}",
                i, length, i, i
            );
        }
        for i in 17..=50 {
            let part_1 = ((0xF0 & i) >> 4) + 0x80;
            let part_2 = 0x0F & i;
            let part_1 = format!("{:#04x}", part_1);
            let part_2 = format!("{:#04x}", part_2);
            println!(
                "rule string{}() -> Vec<u8> = [{}] [{}] n:(any_number())*<{},{}> {{ n }}",
                i - 2,
                part_1,
                part_2,
                i - 2,
                i - 2
            );
        }

        let strings = (1..49)
            .map(|x| format!("string{}()", x))
            .fold(String::from("string0()"), |a, b| format!("{}/{}", a, b));

        let string_rule = format!("pub rule string = {}", strings);

        println!("{}", string_rule);
    }

    #[test]
    pub fn generate_sml_response_body() {
        for i in 1..=15 {
            let length = format!("{:#04x}", i + 0x70);
            println!(
                "rule list_sml_value{}() -> Vec<SmlListEntry> = [{}] n:(single_sml_value())*<{},{}> {{ n }}",
                i, length, i, i
            );
        }

        let strings = (1..16)
            .map(|x| format!("list_sml_value{}()", x))
            .fold(String::from("list_sml_value1()"), |a, b| {
                format!("{}/{}", a, b)
            });

        println!("rule list_sml_value() -> Vec<SmlListEntry> = {}", strings);
    }
}
