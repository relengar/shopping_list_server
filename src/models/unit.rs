use serde_derive::{Deserialize, Serialize};
use std::str::{FromStr};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Unit {
    ITEM,
    TEASPOON,
    TABLESPOON,
    FLUID,
    GILL,
    CUP,
    PINT,
    QUART,
    GALLON,
    MILLILITER,
    LITER,
    DECILITER,
    POUND,
    OUNCE,
    MILLIGRAM,
    GRAM,
    KILOGRAM,
    MILLIMETER,
    CENTIMETER,
    METER,
    INCH,
}

impl FromStr for Unit {
    type Err = ();

    fn from_str(input: &str) -> Result<Unit, Self::Err> {
        match input {
            "ITEM" => Ok(Unit::ITEM),
            "TEASPOON" => Ok(Unit::TEASPOON),
            "TABLESPOON" => Ok(Unit::TABLESPOON),
            "FLUID" => Ok(Unit::FLUID),
            "GILL" => Ok(Unit::GILL),
            "CUP" => Ok(Unit::CUP),
            "PINT" => Ok(Unit::PINT),
            "QUART" => Ok(Unit::QUART),
            "GALLON" => Ok(Unit::GALLON),
            "MILLILITER" => Ok(Unit::MILLILITER),
            "LITER" => Ok(Unit::LITER),
            "DECILITER" => Ok(Unit::DECILITER),
            "POUND" => Ok(Unit::POUND),
            "OUNCE" => Ok(Unit::OUNCE),
            "MILLIGRAM" => Ok(Unit::MILLIGRAM),
            "GRAM" => Ok(Unit::GRAM),
            "KILOGRAM" => Ok(Unit::KILOGRAM),
            "MILLIMETER" => Ok(Unit::MILLIMETER),
            "CENTIMETER" => Ok(Unit::CENTIMETER),
            "METER" => Ok(Unit::METER),
            "INCH" => Ok(Unit::INCH),
            _      => Err(()),
        }
    }
}

impl ToString for Unit {
    fn to_string(self: &Self) -> String {
        match self {
            Unit::ITEM => String::from("ITEM"),
            Unit::TEASPOON => String::from("TEASPOON"),
            Unit::TABLESPOON => String::from("TABLESPOON"),
            Unit::FLUID => String::from("FLUID"),
            Unit::GILL => String::from("GILL"),
            Unit::CUP => String::from("CUP"),
            Unit::PINT => String::from("PINT"),
            Unit::QUART => String::from("QUART"),
            Unit::GALLON => String::from("GALLON"),
            Unit::MILLILITER => String::from("MILLILITER"),
            Unit::LITER => String::from("LITER"),
            Unit::DECILITER => String::from("DECILITER"),
            Unit::POUND => String::from("POUND"),
            Unit::OUNCE => String::from("OUNCE"),
            Unit::MILLIGRAM => String::from("MILLIGRAM"),
            Unit::GRAM => String::from("GRAM"),
            Unit::KILOGRAM => String::from("KILOGRAM"),
            Unit::MILLIMETER => String::from("MILLIMETER"),
            Unit::CENTIMETER => String::from("CENTIMETER"),
            Unit::METER => String::from("METER"),
            Unit::INCH => String::from("INCH"),
        }
    }
}
