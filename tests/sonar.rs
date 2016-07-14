use clang::*;

pub fn test(clang: &Clang) {
    macro_rules! assert_declaration_eq {
        ($declaration:expr, $name:expr, SAME) => ({
            let declaration = $declaration;
            assert_eq!(declaration.name, $name);
            assert_eq!(declaration.entity.get_name(), Some($name.into()));
            assert!(declaration.source.is_none());
        });

        ($declaration:expr, $name:expr, DIFFERENT) => ({
            let declaration = $declaration;
            assert_eq!(declaration.name, $name);
            assert_eq!(declaration.entity.get_name(), None);
            assert_eq!(declaration.source.unwrap().get_name(), Some($name.into()));
        });
    }

    let source = "
        #define A 4
        #define B -322
        #define C 3.14159
        #define D -2.71828
    ";

    super::with_temporary_file("header.h", source, |_, f| {
        use clang::sonar::{DefinitionValue};

        let index = Index::new(&clang, false, false);
        let tu = index.parser(f).detailed_preprocessing_record(true).parse().unwrap();

        let definitions = sonar::find_definitions(tu.get_entity().get_children()).filter(|d| {
            !d.entity.is_in_system_header()
        }).collect::<Vec<_>>();
        assert_eq!(definitions.len(), 4);

        macro_rules! assert_definition_eq {
            ($definition:expr, $name:expr, $value:expr) => ({
                let definition = $definition;
                assert_eq!(definition.name, $name);
                assert_eq!(definition.value, $value);
                assert_eq!(definition.entity.get_name(), Some($name.into()));
            });
        }

        assert_definition_eq!(&definitions[0], "A", DefinitionValue::Integer(false, 4));
        assert_definition_eq!(&definitions[1], "B", DefinitionValue::Integer(true, 322));
        assert_definition_eq!(&definitions[2], "C", DefinitionValue::Real(3.14159));
        assert_definition_eq!(&definitions[3], "D", DefinitionValue::Real(-2.71828));
    });

    let source = "
        enum A {
            AA = 1,
            AB = 2,
            AC = 3,
        };

        typedef enum B {
            CA,
            CB,
            CC,
        } B;

        typedef enum {
            DA,
            DB,
            DC,
        } C;

        enum D {
            EA,
            EB,
            EC,
        };

        typedef enum D D;

        typedef enum E* OpaqueE;
    ";

    super::with_entity(&clang, source, |e| {
        assert_eq!(sonar::find_typedefs(e.get_children()).count(), 1);

        let enums = sonar::find_enums(e.get_children()).filter(|e| {
            !e.entity.is_in_system_header()
        }).collect::<Vec<_>>();
        assert_eq!(enums.len(), 4);

        assert_declaration_eq!(&enums[0], "A", SAME);
        assert_declaration_eq!(&enums[1], "B", SAME);
        assert_declaration_eq!(&enums[2], "C", DIFFERENT);
        assert_declaration_eq!(&enums[3], "D", SAME);
    });

    let source = "
        void multiple(void);
        void multiple(void);

        int zero(void);

        float one(int a);
        float two(int a, int b);

        double many(int a, int b, ...);
    ";

    super::with_entity(&clang, source, |e| {
        let functions = sonar::find_functions(e.get_children()).filter(|f| {
            !f.entity.is_in_system_header()
        }).collect::<Vec<_>>();
        assert_eq!(functions.len(), 5);

        assert_declaration_eq!(&functions[0], "multiple", SAME);
        assert_declaration_eq!(&functions[1], "zero", SAME);
        assert_declaration_eq!(&functions[2], "one", SAME);
        assert_declaration_eq!(&functions[3], "two", SAME);
        assert_declaration_eq!(&functions[4], "many", SAME);
    });

    let source = "
        struct A {
            int a;
        };

        typedef struct B {
            int b;
        } B;

        typedef struct {
            int c;
        } C;

        struct D {
            int d;
        };

        typedef struct D D;

        typedef struct E* OpaqueE;
    ";

    super::with_entity(&clang, source, |e| {
        assert_eq!(sonar::find_typedefs(e.get_children()).count(), 1);

        let structs = sonar::find_structs(e.get_children()).filter(|s| {
            !s.entity.is_in_system_header()
        }).collect::<Vec<_>>();
        assert_eq!(structs.len(), 4);

        assert_declaration_eq!(&structs[0], "A", SAME);
        assert_declaration_eq!(&structs[1], "B", SAME);
        assert_declaration_eq!(&structs[2], "C", DIFFERENT);
        assert_declaration_eq!(&structs[3], "D", SAME);
    });

    let source = "
        typedef int Integer;
        typedef Integer IntegerTypedef;
        typedef IntegerTypedef IntegerTypedefTypedef;

        typedef int* IntegerPointer;

        typedef int Function(int, float, double);
        typedef int (*FunctionPointer)(int a, float b, double c, ...);

        enum E { EA, EB, EC };
        typedef enum E Enum;
        typedef Enum EnumTypedef;

        struct S { int s; };
        typedef struct S Struct;
        typedef Struct StructTypedef;

        union U { int us; float uf; };
        typedef union U Union;
        typedef Union UnionTypedef;

        typedef enum E EArray[4];
        typedef struct S SArray[4];
        typedef union U UArray[4];

        typedef enum Enum_* OpaqueEnum;
        typedef struct Struct_* OpaqueStruct;
        typedef Union_* OpaqueUnion;
    ";

    super::with_entity(&clang, source, |e| {
        assert_eq!(sonar::find_enums(e.get_children()).count(), 1);
        assert_eq!(sonar::find_structs(e.get_children()).count(), 1);
        assert_eq!(sonar::find_unions(e.get_children()).count(), 1);

        let typedefs = sonar::find_typedefs(e.get_children()).filter(|t| {
            !t.entity.is_in_system_header()
        }).collect::<Vec<_>>();
        assert_eq!(typedefs.len(), 18);

        assert_declaration_eq!(&typedefs[0], "Integer", SAME);
        assert_declaration_eq!(&typedefs[1], "IntegerTypedef", SAME);
        assert_declaration_eq!(&typedefs[2], "IntegerTypedefTypedef", SAME);
        assert_declaration_eq!(&typedefs[3], "IntegerPointer", SAME);
        assert_declaration_eq!(&typedefs[4], "Function", SAME);
        assert_declaration_eq!(&typedefs[5], "FunctionPointer", SAME);
        assert_declaration_eq!(&typedefs[6], "Enum", SAME);
        assert_declaration_eq!(&typedefs[7], "EnumTypedef", SAME);
        assert_declaration_eq!(&typedefs[8], "Struct", SAME);
        assert_declaration_eq!(&typedefs[9], "StructTypedef", SAME);
        assert_declaration_eq!(&typedefs[10], "Union", SAME);
        assert_declaration_eq!(&typedefs[11], "UnionTypedef", SAME);
        assert_declaration_eq!(&typedefs[12], "EArray", SAME);
        assert_declaration_eq!(&typedefs[13], "SArray", SAME);
        assert_declaration_eq!(&typedefs[14], "UArray", SAME);
        assert_declaration_eq!(&typedefs[15], "OpaqueEnum", SAME);
        assert_declaration_eq!(&typedefs[16], "OpaqueStruct", SAME);
        assert_declaration_eq!(&typedefs[17], "OpaqueUnion", SAME);
    });

    let source = "
        union A {
            int ai;
            float af;
        };

        typedef union B {
            int bi;
            float bf;
        } B;

        typedef union {
            int ci;
            float cf;
        } C;

        union D {
            int di;
            float df;
        };

        typedef union D D;

        typedef union E* OpaqueE;
    ";

    super::with_entity(&clang, source, |e| {
        assert_eq!(sonar::find_typedefs(e.get_children()).count(), 1);

        let unions = sonar::find_unions(e.get_children()).filter(|u| {
            !u.entity.is_in_system_header()
        }).collect::<Vec<_>>();
        assert_eq!(unions.len(), 4);

        assert_declaration_eq!(&unions[0], "A", SAME);
        assert_declaration_eq!(&unions[1], "B", SAME);
        assert_declaration_eq!(&unions[2], "C", DIFFERENT);
        assert_declaration_eq!(&unions[3], "D", SAME);
    });

    #[cfg(target_os="linux")]
    fn test_headers(clang: &Clang) {
        fn test(clang: &Clang, header: &str) {
            let index = Index::new(clang, false, false);
            let tu = index.parser(header).detailed_preprocessing_record(true).parse();
            if let Ok(tu) = tu {
                let entities = tu.get_entity().get_children();
                let _ = sonar::find_definitions(&entities[..]);
                let _ = sonar::find_enums(&entities[..]);
                let _ = sonar::find_functions(&entities[..]);
                let _ = sonar::find_structs(&entities[..]);
                let _ = sonar::find_typedefs(&entities[..]);
                let _ = sonar::find_unions(&entities[..]);
            }
        }

        test(clang, "/usr/include/assert.h");
        test(clang, "/usr/include/complex.h");
        test(clang, "/usr/include/ctype.h");
        test(clang, "/usr/include/errno.h");
        test(clang, "/usr/include/fenv.h");
        test(clang, "/usr/include/float.h");
        test(clang, "/usr/include/inttypes.h");
        test(clang, "/usr/include/limits.h");
        test(clang, "/usr/include/locale.h");
        test(clang, "/usr/include/math.h");
        test(clang, "/usr/include/setjmp.h");
        test(clang, "/usr/include/signal.h");
        test(clang, "/usr/include/stdarg.h");
        test(clang, "/usr/include/stdbool.h");
        test(clang, "/usr/include/stddef.h");
        test(clang, "/usr/include/stdint.h");
        test(clang, "/usr/include/stdio.h");
        test(clang, "/usr/include/stdlib.h");
        test(clang, "/usr/include/string.h");
        test(clang, "/usr/include/tgmath.h");
        test(clang, "/usr/include/time.h");
        test(clang, "/usr/include/uchar.h");
        test(clang, "/usr/include/wchar.h");
        test(clang, "/usr/include/wctype.h");
    }

    #[cfg(not(target_os="linux"))]
    fn test_headers(_: &Clang) { }

    test_headers(clang);
}
