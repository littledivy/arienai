macro_rules! repr_u8 {
  ($(#[$meta:meta])* $vis:vis enum $name:ident {
    $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
  }) => {
    $(#[$meta])*
    $vis enum $name {
      $($(#[$vmeta])* $vname $(= $val)?,)*
    }

    impl core::convert::TryFrom<u8> for $name {
      type Error = ();

      fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
          $(x if x == $name::$vname as u8 => Ok($name::$vname),)*
          _ => Err(()),
        }
      }
    }
  }
}

repr_u8! {
  #[repr(u8)]
  pub enum Message {
    Sign = 0x00,
    Verify = 0x01,
    GetOwner = 0x02,
    GetAddress = 0x03,
  }
}
