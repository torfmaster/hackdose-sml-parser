// cf. https://www.promotic.eu/en/pmdoc/Subsystems/Comm/PmDrivers/IEC62056_OBIS.htm

macro_rules! generate_obis {

     ($( ($x:ident, $y:expr, $l:literal) ),*) => {
        #[non_exhaustive]
        pub enum Obis {
             $(
                #[doc = $l]
                 $x,
             )*
         }

        impl Obis {
             pub fn obis_number(&self) -> &[u8] {
                 match self {
                    $(
                        Self:: $x => $y,
                    )*
                 }
             }
         }
    };
}

generate_obis! {
    (SumActiveInstantaneousPower, &[1, 0, 16, 7, 0, 255], "Sum active energy without reverse blockade (A+ - A-) total [kWh]"),
    (PositiveActiveEnergy, &[1, 0, 1, 8, 0, 255], "Positive active energy (A+) total [kWh]"),
    (PositiveActiveEnergyTarif1, &[1, 0, 1, 8, 1, 255], "Positive active energy (A+) in tariff T1 [kWh]"),
    (NegativeActiveEnergyTotal , &[1, 0, 2, 8, 0, 255], "Negative active energy (A+) total [kWh]")
}
