use core::{fmt, fmt::{Debug,Formatter}};

pub enum ConvertError {
	CantConvertFrom,
	CantConvertTo,
	CantConvertAmount,
}

impl Debug for ConvertError {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			ConvertError::CantConvertFrom => write!(f, "Cant convert from field"),
			ConvertError::CantConvertTo => write!(f, "Cant convert to field"),
			ConvertError::CantConvertAmount => write!(f, "Cant convert amount field")
		}
	}
}
