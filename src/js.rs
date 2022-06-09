use std::fmt::{Debug, Display};

pub struct JS(String);

impl Display for JS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for JS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for JS {
    fn from(s: &str) -> Self {
        JS(Self::transform(s.to_string()))
    }
}

impl From<String> for JS {
    fn from(s: String) -> Self {
        JS(Self::transform(s))
    }
}

impl JS {
    fn transform(mut s: String) -> String {
        let (from, to) = ([b'\"', b'(', b')'], [b'\'', b'[', b']']);

        let bytes = unsafe { s.as_bytes_mut() };
        for b in bytes {
            if let Some(idx) = from.iter().position(|c| c == b) {
                *b = to[idx] as u8;
            }
        }
        s
    }

    fn peek_next(bytes: &[u8]) -> Option<u8> {
        if let Some(idx) = bytes.iter().position(|b| !b.is_ascii_whitespace()) {
            Some(bytes[idx])
        } else {
            None
        }
    }

    pub fn dump(self) -> String {
        self.0
    }

    pub fn pretty(self) -> String {
        let mut bytes = self.0.into_bytes();
        let (mut i, mut indent) = (0, 0);

        let tab = |indent| format!("\n{}", " ".repeat(indent * 4usize));
        let apply_indent = |bytes: &mut Vec<u8>, at: &mut usize, prefix: String| {
            let jump = prefix.len();
            bytes.splice(*at..*at + 1, prefix.into_bytes());
            *at += jump;
        };
        let is_complex = |bytes: &[u8]| bytes.iter().any(|&b| b == b':' || b == b'{' || b == b'[');

        while i < bytes.len() {
            match bytes[i] {
                b'[' => {
                    let j = i + bytes[i..].iter().position(|&c| c == b']').unwrap();
                    if is_complex(&bytes[i + 1..j]) {
                        indent += 1;
                        apply_indent(&mut bytes, &mut i, format!("[{}", tab(indent)));
                    } else {
                        i = j + 1;
                    }
                }
                b',' => {
                    i += 1;
                    match JS::peek_next(&bytes[i..]) {
                        Some(b'}') | None => (),
                        _ => apply_indent(&mut bytes, &mut i, tab(indent)),
                    };
                }
                b'{' => {
                    indent += 1;
                    apply_indent(&mut bytes, &mut i, format!("{{{}", tab(indent)));
                }
                b'}' => {
                    indent -= 1;
                    apply_indent(&mut bytes, &mut i, format!("{}}}", tab(indent)));
                }
                b']' => {
                    indent -= 1;
                    apply_indent(&mut bytes, &mut i, format!("{}]", tab(indent)));
                }
                _ => i += 1,
            }
        }

        bytes.into_iter().map(|b| b as char).collect()
    }
}

#[macro_export]
macro_rules! js {
    // Handle single keywords, literals and identifiers
    () => { $crate::js::JS::from("") };
    (null) => { $crate::js::JS::from("null") };
    (undefined) => { $crate::js::JS::from("undefined") };
    ($value:literal) => { $crate::js::JS::from(stringify!($value)) };
    ($value:ident) => { $crate::js::JS::from(format!("{:?}", $value)) };

    // Handle simple collections
    ($($value:literal),*) => {
        $crate::js::JS::from(vec![$( stringify!($value), )*].join(", "))
    };
    ($($value:ident),*) => {
        $crate::js::JS::from(vec![$( format!("{:?}", $value), )*].join(", "))
    };

    // Handle trailing expressions
    ($key:ident : ($value:expr)) => {
        $crate::js::JS::from(format!("{}: {:?}", stringify!($key), $value))
    };
    ($key:ident : ($value:expr)?) => {
        if $value.is_some() {
            $crate::js::JS::from(format!("{}: {:?}", stringify!($key), $value))
        } else {
            $crate::js::JS::from(format!("{}: undefined", stringify!($key)))
        }
    };
    ($key:ident : $value:tt) => {{
        $crate::js::JS::from(format!("{}: {}", stringify!($key), js!($value)))
    }};
    ($key:ident : -$value:tt) => {{
        $crate::js::JS::from(format!("{}: {}", stringify!($key), js!(-$value)))
    }};

    // Handle trailing nested expressions
    ($key:ident : { $($body:tt)* }) => {
        $crate::js::JS::from(format!("{}: {{{}}}", stringify!($key), js!($($body)*)))
    };
    ({ $($body:tt)* }) => {
        $crate::js::JS::from(format!("{{{}}}", js!($($body)*)))
    };
    ($key:ident : [ $($body:tt)* ]) => {
        $crate::js::JS::from(format!("{}: [{}]", stringify!($key), js!($($body)*)))
    };
    ([ $($body:tt)* ]) => {
        $crate::js::JS::from(format!("[{}]", js!($($body)*)))
    };

    // Handle consecutive expressions
    ($key:ident : ($value:expr), $($tail:tt)*) => {
        $crate::js::JS::from(format!("{}: {:?}, {}", stringify!($key), $value, js!($($tail)*)))
    };
    ($key:ident : ($value:expr)?, $($tail:tt)*) => {
        if $value.is_some() {
            $crate::js::JS::from(format!("{}: {:?}, {}", stringify!($key), $value.unwrap(), js!($($tail)*)))
        } else {
            $crate::js::JS::from(format!("{}: undefined, {}", stringify!($key), js!($($tail)*)))
        }
    };
    ($key:ident : $value:tt, $($tail:tt)*) => {
        $crate::js::JS::from(format!("{}: {}, {}", stringify!($key), js!($value), js!($($tail)*)))
    };
    ($key:ident : -$value:tt, $($tail:tt)*) => {
        $crate::js::JS::from(format!("{}: {}, {}", stringify!($key), js!(-$value), js!($($tail)*)))
    };

    // Handle consecutive nested expressions
    ($key:ident : { $($body:tt)* }, $($tail:tt)*) => {
        $crate::js::JS::from(format!("{}: {{{}}}, {}", stringify!($key),  js!($($body)*), js!($($tail)*)))
    };
    ({ $($body:tt)* }, $($tail:tt)*) => {
        $crate::js::JS::from(format!("{{{}}}, {}", js!($($body)*), js!($($tail)*)))
    };
    ($key:ident : [ $($body:tt)* ], $($tail:tt)*) => {
        $crate::js::JS::from(format!("{}: [{}], {}", stringify!($key),  js!($($body)*), js!($($tail)*)))
    };
    ([ $($body:tt)* ], $($tail:tt)*) => {
        $crate::js::JS::from(format!("[{}], {}", js!($($body)*), js!($($tail)*)))
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn macro_test() {
        let string = "lorem ipsum";
        let number = 42;
        let collection = ("left", 4, String::from("dead"));
        let inception = js! ({ a: 1, b: 2 });
        assert_eq!("null", js!(null).dump());
        assert_eq!("a: 1", js!(a: 1).dump());
        assert_eq!("a: 1, b: 1", js!(a: 1, b: 1).dump());
        assert_eq!("{a: 1, b: [1, 2]}", js!({a: 1, b: [1, 2]}).dump());

        assert_eq!("string: 'lorem ipsum'", js!(string: string).dump());
        assert_eq!("number: 42", js!(number: number).dump());
        assert_eq!(
            "collection: ['left', 4, 'dead']",
            js!(collection: collection).dump()
        );
        assert_eq!("inception: {a: 1, b: 2}", js!(inception: inception).dump());

        assert_eq!(
            "opt: {list: [{a: 42, b: true}, undefined]}",
            js!(opt: {list: [{a: number, b: true}, undefined]}).dump()
        );
    }

    #[test]
    fn pretty_test() {
        let js = js! {
            {
                a: null,
                b: [
                    {
                        x: 12.2,
                        y: -32.4
                    },
                    {
                        x: 0,
                        y: undefined
                    }
                ],
                c: [1, 2, 3],
                d: "lorem ipsum"
            }
        };
        #[rustfmt::skip]
        let pretty_string =
"{
    a: null,
    b: [
        {
            x: 12.2,
            y: -32.4
        },
        {
            x: 0,
            y: undefined
        }
    ],
    c: [1, 2, 3],
    d: 'lorem ipsum'
}";

        assert_eq!(js.pretty(), pretty_string);
    }
}
