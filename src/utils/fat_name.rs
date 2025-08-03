use alloc::string::String;

pub fn fat_name_from_chars(chars: &[u8; 11]) -> String {
    // TODO add error checking
    let mut name = String::new();

    for i in 0..8 {
        let char = chars[i] as char;
        if char.is_alphanumeric() {
            name.push(char);
        }
    }

    name.push('.');

    for i in 8..11 {
        let char = chars[i] as char;

        if char.is_alphanumeric() {
            name.push(char);
        }
    }

    return name;
}

#[cfg(test)]
mod tests {
    struct NameParseTest {
        bytes: [u8; 11],
        name: &'static str
    }

    impl NameParseTest {
        pub const fn expect_is(bytes: [u8; 11], name: &'static str) -> Self {
            Self { bytes, name }
        }

        pub fn run_test(&self) {
            assert_eq!(super::fat_name_from_chars(&self.bytes), self.name);
        }
    }

    const FOO_BAR_TEST: NameParseTest = NameParseTest::expect_is(
        [b'F', b'O', b'O', b' ', b' ', b' ', b' ', b' ', b'B', b'A', b'R'],
        "FOO.BAR"
    );   

    const FOO_TEST: NameParseTest = NameParseTest::expect_is(
        [b'F', b'O', b'O', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' '],
        "FOO."
    );

    const PICKLE_TEST: NameParseTest = NameParseTest::expect_is(
        [b'P', b'I', b'C', b'K', b'L', b'E', b' ', b' ', b'A', b' ', b' '],
        "PICKLE.A"
    );

    const PRETTY_BIG_TEST: NameParseTest = NameParseTest::expect_is(
        [b'P', b'R', b'E', b'T', b'T', b'Y', b'B', b'G', b'B', b'I', b'G'],
        "PRETTYBG.BIG"
    );

    const DOT_BIG_TEST: NameParseTest = NameParseTest::expect_is(
        [b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'B', b'I', b'G'],
        ".BIG"
    );


    #[test]
    fn test_foo_bar() {
        FOO_BAR_TEST.run_test();
    } 

    #[test]
    fn test_foo() {
        FOO_TEST.run_test();
    }

    #[test]
    fn test_pickle() {
        PICKLE_TEST.run_test();
    }

    #[test]
    fn test_pretty_big() {
        PRETTY_BIG_TEST.run_test();
    }

    #[test]
    fn test_dot_big() {
        DOT_BIG_TEST.run_test();
    }
}