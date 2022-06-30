//! # xrust::value
//!
//! An atomic value as an item in a sequence.

use core::fmt;
use std::convert::TryFrom;
use std::cmp::Ordering;
use rust_decimal::Decimal;
#[cfg(test)]
use rust_decimal_macros::dec;
use chrono::{Date, DateTime, Local};
use crate::xdmerror::{Error, ErrorKind};

/// Comparison operators for values
#[derive(Copy, Clone)]
pub enum Operator {
  Equal,
  NotEqual,
  LessThan,
  LessThanEqual,
  GreaterThan,
  GreaterThanEqual,
  Is,
  Before,
  After,
}

impl Operator {
  pub fn to_string(&self) -> &str {
    match self {
      Operator::Equal => "=",
      Operator::NotEqual => "!=",
      Operator::LessThan => "<",
      Operator::LessThanEqual => "<=",
      Operator::GreaterThan => ">",
      Operator::GreaterThanEqual => ">=",
      Operator::Is => "is",
      Operator::Before => "<<",
      Operator::After => ">>",
    }
  }
}

/// A concrete type that implements atomic values.
/// These are the 19 predefined types in XSD Schema Part 2, plus five additional types.
#[derive(Clone)]
pub enum Value {
    /// node or simple type
    AnyType,
    /// a not-yet-validated anyType
    Untyped,
    /// base type of all simple types. i.e. not a node
    AnySimpleType,
    /// a list of IDREF
    IDREFS,
    /// a list of NMTOKEN
    NMTOKENS,
    /// a list of ENTITY
    ENTITIES,
    /// Any numeric type
    Numeric,
    /// all atomic values (no lists or unions)
    AnyAtomicType,
    /// untyped atomic value
    UntypedAtomic,
    Duration,
    Time(DateTime<Local>),	// Ignore the date part. Perhaps use Instant instead?
    Decimal(Decimal),
    Float(f32),
    Double(f64),
    Integer(i64),
    NonPositiveInteger(NonPositiveInteger),
    NegativeInteger(NegativeInteger),
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    NonNegativeInteger(NonNegativeInteger),
    UnsignedLong(u64),
    UnsignedInt(u32),
    UnsignedShort(u16),
    UnsignedByte(u8),
    PositiveInteger(PositiveInteger),
    DateTime(DateTime<Local>),
    DateTimeStamp,
    Date(Date<Local>),
    String(String),
    NormalizedString(NormalizedString),
    /// Like normalizedString, but without leading, trailing and consecutive whitespace
    Token,
    /// language identifiers [a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*
    Language,
    /// NameChar+
    NMTOKEN,
    /// NameStartChar NameChar+
    Name,
    /// (Letter | '_') NCNameChar+ (i.e. a Name without the colon)
    NCName,
    /// Same format as NCName
    ID,
    /// Same format as NCName
    IDREF,
    /// Same format as NCName
    ENTITY,
    Boolean(bool),
}

impl Value {
    /// Give the string value.
    pub fn to_string(&self) -> String {
	match self {
	    Value::String(s) => s.to_string(),
	    Value::NormalizedString(s) => s.0.to_string(),
	    Value::Decimal(d) => d.to_string(),
	    Value::Float(f) => f.to_string(),
	    Value::Double(d) => d.to_string(),
	    Value::Integer(i) => i.to_string(),
	    Value::Long(l) => l.to_string(),
	    Value::Short(s) => s.to_string(),
	    Value::Int(i) => i.to_string(),
	    Value::Byte(b) => b.to_string(),
	    Value::UnsignedLong(l) => l.to_string(),
	    Value::UnsignedShort(s) => s.to_string(),
	    Value::UnsignedInt(i) => i.to_string(),
	    Value::UnsignedByte(b) => b.to_string(),
	    Value::NonPositiveInteger(i) => i.0.to_string(),
	    Value::NonNegativeInteger(i) => i.0.to_string(),
	    Value::PositiveInteger(i) => i.0.to_string(),
	    Value::NegativeInteger(i) => i.0.to_string(),
	    Value::Time(t) => t.format("%H:%M:%S.%f").to_string(),
	    Value::DateTime(dt) => dt.format("%Y-%m-%dT%H:%M:%S%z").to_string(),
	    Value::Date(d) => d.format("%Y-%m-%d").to_string(),
 	    _ => "".to_string(),
	}
    }

    /// Give the effective boolean value.
    pub fn to_bool(&self) -> bool {
	match &self {
            Value::Boolean(b) => *b == true,
            Value::String(t) => {
                //t.is_empty()
	        t.len() != 0
            },
	    Value::NormalizedString(s) => s.0.len() != 0,
            Value::Double(n) => *n != 0.0,
            Value::Integer(i) => *i != 0,
            Value::Int(i) => *i != 0,
            _ => false
	}
    }

    /// Convert the value to an integer, if possible.
    pub fn to_int(&self) -> Result<i64, Error> {
        match &self {
            Value::Int(i) => Ok(*i as i64),
	    Value::Integer(i) => Ok(*i),
	    _ => {
	      match self.to_string().parse::<i64>() {
	        Ok(i) => Ok(i),
		Err(e) => Result::Err(Error{kind: ErrorKind::Unknown, message: format!("type conversion error: {}", e)}),
	      }
	    }
	}
    }
    /// Convert the value to a double. If the value cannot be converted, returns Nan.
    pub fn to_double(&self) -> f64 {
        match &self {
	    Value::String(s) => {
	      match s.parse::<f64>() {
	        Ok(i) => i,
		Err(_) => f64::NAN,
	      }
	    }
            Value::Integer(i) => (*i) as f64,
            Value::Double(d) => *d,
            _ => f64::NAN,
	}
    }
    pub fn value_type(&self) -> &'static str {
      match &self {
        Value::AnyType => "AnyType",
        Value::Untyped => "Untyped",
        Value::AnySimpleType => "AnySimpleType",
        Value::IDREFS => "IDREFS",
        Value::NMTOKENS => "NMTOKENS",
        Value::ENTITIES => "ENTITIES",
        Value::Numeric => "Numeric",
        Value::AnyAtomicType => "AnyAtomicType",
        Value::UntypedAtomic => "UntypedAtomic",
        Value::Duration => "Duration",
        Value::Time(_) => "Time",
        Value::Decimal(_) => "Decimal",
        Value::Float(_) => "Float",
        Value::Double(_) => "Double",
        Value::Integer(_) => "Integer",
        Value::NonPositiveInteger(_) => "NonPositiveInteger",
        Value::NegativeInteger(_) => "NegativeInteger",
        Value::Long(_) => "Long",
        Value::Int(_) => "Int",
        Value::Short(_) => "Short",
        Value::Byte(_) => "Byte",
        Value::NonNegativeInteger(_) => "NonNegativeInteger",
        Value::UnsignedLong(_) => "UnsignedLong",
        Value::UnsignedInt(_) => "UnsignedInt",
        Value::UnsignedShort(_) => "UnsignedShort",
        Value::UnsignedByte(_) => "UnsignedByte",
        Value::PositiveInteger(_) => "PositiveInteger",
        Value::DateTime(_) => "DateTime",
        Value::DateTimeStamp => "DateTimeStamp",
        Value::Date(_) => "Date",
        Value::String(_) => "String",
        Value::NormalizedString(_) => "NormalizedString",
        Value::Token => "Token",
        Value::Language => "Language",
        Value::NMTOKEN => "NMTOKEN",
        Value::Name => "Name",
        Value::NCName => "NCName",
        Value::ID => "ID",
        Value::IDREF => "IDREF",
        Value::ENTITY => "ENTITY",
	Value::Boolean(_) => "boolean",
      }
    }
    pub fn compare(&self, other: &Value, op: Operator) -> Result<bool, Error> {
	match &self {
	    Value::Boolean(b) => {
		let c = other.to_bool();
		match op {
		    Operator::Equal => Ok(*b == c),
		    Operator::NotEqual => Ok(*b != c),
		    Operator::LessThan => Ok(*b < c),
		    Operator::LessThanEqual => Ok(*b <= c),
		    Operator::GreaterThan => Ok(*b > c),
		    Operator::GreaterThanEqual => Ok(*b >= c),
		    Operator::Is |
		    Operator::Before |
		    Operator::After => Result::Err(Error::new(ErrorKind::TypeError, String::from("type error"))),
		}
	    }
	    Value::Integer(i) => {
		let c = other.to_int()?;
		match op {
		    Operator::Equal => Ok(*i == c),
		    Operator::NotEqual => Ok(*i != c),
		    Operator::LessThan => Ok(*i < c),
		    Operator::LessThanEqual => Ok(*i <= c),
		    Operator::GreaterThan => Ok(*i > c),
		    Operator::GreaterThanEqual => Ok(*i >= c),
		    Operator::Is |
		    Operator::Before |
		    Operator::After => Result::Err(Error::new(ErrorKind::TypeError, String::from("type error"))),
		}
	    }
	    Value::Double(i) => {
		let c = other.to_double();
		match op {
		    Operator::Equal => Ok(*i == c),
		    Operator::NotEqual => Ok(*i != c),
		    Operator::LessThan => Ok(*i < c),
		    Operator::LessThanEqual => Ok(*i <= c),
		    Operator::GreaterThan => Ok(*i > c),
		    Operator::GreaterThanEqual => Ok(*i >= c),
		    Operator::Is |
		    Operator::Before |
		    Operator::After => Result::Err(Error::new(ErrorKind::TypeError, String::from("type error"))),
		}
	    }
	    Value::String(i) => {
		let c = other.to_string();
		match op {
		    Operator::Equal => Ok(*i == c),
		    Operator::NotEqual => Ok(*i != c),
		    Operator::LessThan => Ok(*i < c),
		    Operator::LessThanEqual => Ok(*i <= c),
		    Operator::GreaterThan => Ok(*i > c),
		    Operator::GreaterThanEqual => Ok(*i >= c),
		    Operator::Is |
		    Operator::Before |
		    Operator::After => Result::Err(Error::new(ErrorKind::TypeError, String::from("type error"))),
		}
	    }
	    _ => Result::Err(Error::new(ErrorKind::Unknown, format!("comparing type \"{}\" is not yet implemented", self.value_type())))
	}
    }
}

impl PartialEq for Value {
  fn eq(&self, other: &Value) -> bool {
    match self {
        Value::String(s) => s.eq(&other.to_string()),
	Value::Boolean(b) => match other {
	  Value::Boolean(c) => b == c,
	  _ => false, // type error?
	},
	Value::Decimal(d) => match other {
	  Value::Decimal(e) => d == e,
	  _ => false, // type error?
	},
	Value::Integer(i) => match other {
	  Value::Integer(j) => i == j,
	  _ => false, // type error? coerce to integer?
	},
	Value::Double(d) => match other {
	  Value::Double(e) => d == e,
	  _ => false, // type error? coerce to integer?
	},
        _ => false, // not yet implemented
    }
  }
}
impl PartialOrd for Value {
  fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
    match self {
        Value::String(s) => {
	  let o: String = other.to_string();
	  s.partial_cmp(&o)
	},
	Value::Boolean(_) => None,
	Value::Decimal(d) => match other {
	  Value::Decimal(e) => d.partial_cmp(e),
	  _ => None, // type error?
	}
	Value::Integer(d) => match other {
	  Value::Integer(e) => d.partial_cmp(e),
	  _ => None, // type error?
	}
	Value::Double(d) => match other {
	  Value::Double(e) => d.partial_cmp(e),
	  _ => None, // type error?
	}
	_ => None,
    }
  }
}

impl From<String> for Value {
  fn from(s: String) -> Self {
    Value::String(s)
  }
}
impl From<&str> for Value {
  fn from(s: &str) -> Self {
    Value::String(String::from(s))
  }
}
impl From<Decimal> for Value {
  fn from(d: Decimal) -> Self {
    Value::Decimal(d)
  }
}
impl From<f32> for Value {
  fn from(f: f32) -> Self {
    Value::Float(f)
  }
}
impl From<f64> for Value {
  fn from(f: f64) -> Self {
    Value::Double(f)
  }
}
impl From<i64> for Value {
  fn from(i: i64) -> Self {
    Value::Integer(i)
  }
}
impl From<i32> for Value {
  fn from(i: i32) -> Self {
    Value::Int(i)
  }
}
impl From<i16> for Value {
  fn from(i: i16) -> Self {
    Value::Short(i)
  }
}
impl From<i8> for Value {
  fn from(i: i8) -> Self {
    Value::Byte(i)
  }
}
impl From<u64> for Value {
  fn from(i: u64) -> Self {
    Value::UnsignedLong(i)
  }
}
impl From<u32> for Value {
  fn from(i: u32) -> Self {
    Value::UnsignedInt(i)
  }
}
impl From<u16> for Value {
  fn from(i: u16) -> Self {
    Value::UnsignedShort(i)
  }
}
impl From<u8> for Value {
  fn from(i: u8) -> Self {
    Value::UnsignedByte(i)
  }
}
impl From<bool> for Value {
  fn from(b: bool) -> Self {
    Value::Boolean(b)
  }
}

#[derive(Clone, Debug)]
pub struct NonPositiveInteger(i64);
impl TryFrom<i64> for NonPositiveInteger {
  type Error = Error;
  fn try_from(v: i64) -> Result<Self, Self::Error> {
    if v > 0 {
      Err(Error::new(ErrorKind::TypeError, String::from("NonPositiveInteger must be less than zero")))
    } else {
      Ok(NonPositiveInteger(v))
    }
  }
}
impl fmt::Display for NonPositiveInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct PositiveInteger(i64);
impl TryFrom<i64> for PositiveInteger {
  type Error = Error;
  fn try_from(v: i64) -> Result<Self, Self::Error> {
    if v <= 0 {
      Err(Error::new(ErrorKind::TypeError, String::from("PositiveInteger must be greater than zero")))
    } else {
      Ok(PositiveInteger(v))
    }
  }
}
impl fmt::Display for PositiveInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct NonNegativeInteger(i64);
impl TryFrom<i64> for NonNegativeInteger {
  type Error = Error;
  fn try_from(v: i64) -> Result<Self, Self::Error> {
    if v < 0 {
      Err(Error::new(ErrorKind::TypeError, String::from("NonNegativeInteger must be zero or greater")))
    } else {
      Ok(NonNegativeInteger(v))
    }
  }
}
impl fmt::Display for NonNegativeInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct NegativeInteger(i64);
impl TryFrom<i64> for NegativeInteger {
  type Error = Error;
  fn try_from(v: i64) -> Result<Self, Self::Error> {
    if v >= 0 {
      Err(Error::new(ErrorKind::TypeError, String::from("NegativeInteger must be less than zero")))
    } else {
      Ok(NegativeInteger(v))
    }
  }
}
impl fmt::Display for NegativeInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct NormalizedString(String);
impl TryFrom<&str> for NormalizedString {
  type Error = Error;
  fn try_from(v: &str) -> Result<Self, Self::Error> {
    let n: &[_] = &['\n', '\r', '\t'];
    if v.find(n).is_none() {
      Ok(NormalizedString(v.to_string()))
    } else {
      Err(Error::new(ErrorKind::TypeError, String::from("value is not a normalized string")))
    }
  }
}
impl fmt::Display for NormalizedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string() {
        assert_eq!(Value::from(String::from("foobar")).to_string(), "foobar");
    }
    #[test]
    fn from_str() {
        assert_eq!(Value::from("foobar").to_string(), "foobar");
    }
    #[test]
    fn from_decimal() {
        assert_eq!(Value::from(dec!(001.23)).to_string(), "1.23");
    }

    #[test]
    fn normalizedstring_valid_empty() {
        assert_eq!(NormalizedString::try_from("").expect("invalid NormalizedString").0, "");
    }
    #[test]
    fn normalizedstring_valid() {
        assert_eq!(NormalizedString::try_from("notinvalid").expect("invalid NormalizedString").0, "notinvalid");
    }
    #[test]
    fn normalizedstring_valid_spaces() {
        assert_eq!(NormalizedString::try_from("not an invalid string").expect("invalid NormalizedString").0, "not an invalid string");
    }
    #[test]
    fn normalizedstring_invalid_tab() {
        let r = NormalizedString::try_from("contains tab	character");
	assert!(match r {
	    Ok(_) => panic!("string contains tab character"),
	    Err(_) => true,
	})
    }
    #[test]
    fn normalizedstring_invalid_newline() {
        let r = NormalizedString::try_from("contains newline\ncharacter");
	assert!(match r {
	    Ok(_) => panic!("string contains newline character"),
	    Err(_) => true,
	})
    }
    #[test]
    fn normalizedstring_invalid_cr() {
        let r = NormalizedString::try_from("contains carriage return\rcharacter");
	assert!(match r {
	    Ok(_) => panic!("string contains cr character"),
	    Err(_) => true,
	})
    }
    #[test]
    fn normalizedstring_invalid_all() {
        let r = NormalizedString::try_from("contains	all\rforbidden\ncharacters");
	assert!(match r {
	    Ok(_) => panic!("string contains at least one forbidden character"),
	    Err(_) => true,
	})
    }

// Numeric is in the too hard basket for now
//    #[test]
//    fn numeric_float() {
//        assert_eq!(Numeric::new(f32::0.123).value, 0.123);
//    }
//    #[test]
//    fn numeric_double() {
//        assert_eq!(Numeric::new(f64::0.456).value, 0.456);
//    }
//    #[test]
//    fn numeric_decimal() {
//        assert_eq!(Numeric::new(dec!(123.456)), 123.456);
//    }

    #[test]
    fn nonpositiveinteger_valid() {
        assert_eq!(NonPositiveInteger::try_from(-10).expect("invalid NonPositiveInteger").0, -10);
    }
    #[test]
    fn nonpositiveinteger_valid_zero() {
        assert_eq!(NonPositiveInteger::try_from(0).expect("invalid NonPositiveInteger").0, 0);
    }
    #[test]
    fn nonpositiveinteger_invalid() {
        let r = NonPositiveInteger::try_from(10);
	assert!(match r {
	    Ok(_) => panic!("10 is not a nonPositiveInteger"),
	    Err(_) => true,
	})
    }

    #[test]
    fn positiveinteger_valid() {
        assert_eq!(PositiveInteger::try_from(10).expect("invalid PositiveInteger").0, 10);
    }
    #[test]
    fn positiveinteger_invalid_zero() {
        let r = PositiveInteger::try_from(0);
	assert!(match r {
	    Ok(_) => panic!("0 is not a PositiveInteger"),
	    Err(_) => true,
	})
    }
    #[test]
    fn positiveinteger_invalid() {
        let r = PositiveInteger::try_from(-10);
	assert!(match r {
	    Ok(_) => panic!("-10 is not a PositiveInteger"),
	    Err(_) => true,
	})
    }

    #[test]
    fn nonnegativeinteger_valid() {
        assert_eq!(NonNegativeInteger::try_from(10).expect("invalid NonNegativeInteger").0, 10);
    }
    #[test]
    fn nonnegativeinteger_valid_zero() {
        assert_eq!(NonNegativeInteger::try_from(0).expect("invalid NonNegativeInteger").0, 0);
    }
    #[test]
    fn nonnegativeinteger_invalid() {
        let r = NonNegativeInteger::try_from(-10);
	assert!(match r {
	    Ok(_) => panic!("-10 is not a NonNegativeInteger"),
	    Err(_) => true,
	})
    }

    #[test]
    fn negativeinteger_valid() {
        assert_eq!(NegativeInteger::try_from(-10).expect("invalid NegativeInteger").0, -10);
    }
    #[test]
    fn negativeinteger_invalid_zero() {
        let r = NegativeInteger::try_from(0);
	assert!(match r {
	    Ok(_) => panic!("0 is not a NegativeInteger"),
	    Err(_) => true,
	})
    }
    #[test]
    fn negativeinteger_invalid() {
        let r = NegativeInteger::try_from(10);
	assert!(match r {
	    Ok(_) => panic!("10 is not a NegativeInteger"),
	    Err(_) => true,
	})
    }

    // String Values
    #[test]
    fn string_stringvalue() {
        assert_eq!(Value::String("foobar".to_string()).to_string(), "foobar")
    }
    #[test]
    fn decimal_stringvalue() {
        assert_eq!(Value::Decimal(dec!(001.23)).to_string(), "1.23")
    }
    #[test]
    fn float_stringvalue() {
        assert_eq!(Value::Float(001.2300_f32).to_string(), "1.23")
    }
    #[test]
    fn nonpositiveinteger_stringvalue() {
        let npi = NonPositiveInteger::try_from(-00123).expect("invalid nonPositiveInteger");
	let i = Value::NonPositiveInteger(npi);
        assert_eq!(i.to_string(), "-123")
    }
    #[test]
    fn nonnegativeinteger_stringvalue() {
        let nni = NonNegativeInteger::try_from(00123).expect("invalid nonNegativeInteger");
	let i = Value::NonNegativeInteger(nni);
        assert_eq!(i.to_string(), "123")
    }
    #[test]
    fn normalizedstring_stringvalue() {
        let ns = NormalizedString::try_from("foobar").expect("invalid normalizedString");
	let i = Value::NormalizedString(ns);
        assert_eq!(i.to_string(), "foobar")
    }

    // value to_bool

    #[test]
    fn value_to_bool_string() {
      assert_eq!(Value::from("2").to_bool(), true)
    }

    // value to_int

    #[test]
    fn value_to_int_string() {
      assert_eq!(Value::from("2").to_int().expect("cannot convert to integer"), 2)
    }

    // value to_double

    #[test]
    fn value_to_double_string() {
      assert_eq!(Value::from("3.0").to_double(), 3.0)
    }

    // value compare

    #[test]
    fn value_compare_eq() {
      assert_eq!(Value::from("3").compare(&Value::Double(3.0), Operator::Equal).expect("unable to compare"), true)
    }

    #[test]
    fn value_compare_ne() {
      assert_eq!(Value::from("3").compare(&Value::Double(3.0), Operator::NotEqual).expect("unable to compare"), false)
    }

    //#[test]
    //fn value_atomize() {
	//let i = Value::Int(123);
        //assert_eq!(i.atomize().stringvalue(), "123")
    //}

    // Operators
    #[test]
    fn op_equal() {
      assert_eq!(Operator::Equal.to_string(), "=")
    }
    #[test]
    fn op_notequal() {
      assert_eq!(Operator::NotEqual.to_string(), "!=")
    }
    #[test]
    fn op_lt() {
      assert_eq!(Operator::LessThan.to_string(), "<")
    }
    #[test]
    fn op_ltequal() {
      assert_eq!(Operator::LessThanEqual.to_string(), "<=")
    }
    #[test]
    fn op_gt() {
      assert_eq!(Operator::GreaterThan.to_string(), ">")
    }
    #[test]
    fn op_gtequal() {
      assert_eq!(Operator::GreaterThanEqual.to_string(), ">=")
    }
    #[test]
    fn op_is() {
      assert_eq!(Operator::Is.to_string(), "is")
    }
    #[test]
    fn op_before() {
      assert_eq!(Operator::Before.to_string(), "<<")
    }
    #[test]
    fn op_after() {
      assert_eq!(Operator::After.to_string(), ">>")
    }
}

