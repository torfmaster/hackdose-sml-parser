# Hackdose SML-parser

A parser for SML messages as emitted by ISKRA(tm) smart meters for instance.
It currently uses the `peg`-crate to express SML as a parser expression grammar.
It also contains a mapping for OBIS numbers.

# Usage

Once you have obtained the data for your SML-speaking appliance (e.g. a smart meter), you can use the library as follows:

```rust
use hackdose_sml_parser::{
    domain::AnyValue, domain::SmlMessageEnvelope, obis::Obis, parser::parse_body,
};

pub fn find_total_power(body: &[u8]) -> Option<i32> {
    let result = parse_body(body);
    let result = result.ok()?;
    for list in result.messages {
        match list {
            SmlMessageEnvelope::GetOpenResponse(_) => continue,
            SmlMessageEnvelope::GetListResponse(body) => {
                let values = &body.value_list;
                let usage = values.iter().find(|value| {
                    value.object_name == Obis::SumActiveInstantaneousPower.obis_number()
                });

                if let Some(usage) = usage {
                    if let AnyValue::Signed(value) = usage.value {
                        return Some(value as i32);
                    }
                }
            }
            SmlMessageEnvelope::GetCloseResponse => continue,
        }
    }
    return None;
}
```

# Acknowledgements

Most of the work inside the library is actually performed by Kevin Mehall's `peg` crate.
I am also indebted to Stefan Weigert's great [blog post](http://www.stefan-weigert.de/php_loader/sml.php)
about the SML protocol which enabled me to develop the grammar incrementally rather than reading
the whole 80 page [specification](https://www.bsi.bund.de/SharedDocs/Downloads/DE/BSI/Publikationen/TechnischeRichtlinien/TR03109/TR-03109-1_Anlage_Feinspezifikation_Drahtgebundene_LMN-Schnittstelle_Teilb.pdf?__blob=publicationFile) in the first place.

# Contributions

Any contributions are highly appreciated. I published this library (which is part of the
[hackdose](https://github.com/torfmaster/hackdose-server)) to save everyone the work to implement yet another parser for this hard-to-read binary protocol
and to focus on more interesting tasks.

# License

This crate is dual-licensed under MIT and Apache 2 license at your choice.
