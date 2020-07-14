use clang::*;
use clang::completion::*;

pub fn test(clang: &Clang) {
    // CompletionString __________________________

    let source = "
        struct A {
            /// \\brief An integer field.
            int a;
            int b;
            int c;
        };
        void b() { A a; a. }
    ";

    super::with_temporary_file("test.cpp", source, |_, f| {
        let index = Index::new(&clang, false, false);
        let tu = index.parser(f).briefs_in_completion_results(true).parse().unwrap();

        let results = tu.completer(f, 8, 27).briefs(true).complete();
        assert_eq!(results.get_container_kind(), Some((EntityKind::StructDecl, false)));
        assert!(results.get_diagnostics(&tu).is_empty());
        assert_eq!(results.get_usr(), Some(Usr("c:@S@A".into())));

        let context = results.get_context().unwrap();
        assert!(!context.all_types);
        assert!(!context.all_values);
        assert!(!context.class_type_values);
        assert!(context.dot_members);
        assert!(!context.arrow_members);
        assert!(!context.enum_tags);
        assert!(!context.union_tags);
        assert!(!context.struct_tags);
        assert!(!context.class_names);
        assert!(!context.namespaces);
        assert!(!context.nested_name_specifiers);
        assert!(!context.macro_names);
        assert!(!context.natural_language);
        assert!(!context.objc_object_values);
        assert!(!context.objc_selector_values);
        assert!(!context.objc_property_members);
        assert!(!context.objc_interfaces);
        assert!(!context.objc_protocols);
        assert!(!context.objc_categories);
        assert!(!context.objc_instance_messages);
        assert!(!context.objc_class_messages);
        assert!(!context.objc_selector_names);

        if cfg!(feature="clang_6_0") {
            return;
        }

        let mut results = results.get_results();
        if cfg!(target_os="windows") && cfg!(feature="clang_3_8") {
            assert_eq!(results.len(), 7);
        } else {
            assert_eq!(results.len(), 6);
        }
        results.sort();

        macro_rules! assert_result_eq {
            ($result:expr, $kind:expr, $priority:expr, $brief:expr, $parent:expr, $typed:expr, $chunks:expr) => ({
                let result = $result;
                assert_eq!(result.kind, $kind);
                assert_eq!(result.string.get_priority(), $priority);
                assert_eq!(result.string.get_annotations().len(), 0);
                assert_eq!(result.string.get_availability(), Availability::Available);
                assert_eq!(result.string.get_comment_brief(), $brief);
                assert_eq!(result.string.get_parent_name(), Some($parent.into()));
                assert_eq!(result.string.get_typed_text(), Some($typed.into()));
                assert_eq!(result.string.get_chunks(), $chunks);
            });
        }

        assert_result_eq!(results[0], EntityKind::Method, 34, None, "A", "operator=", &[
            CompletionChunk::ResultType("A &".into()),
            CompletionChunk::TypedText("operator=".into()),
            CompletionChunk::LeftParenthesis,
            CompletionChunk::Placeholder("const A &".into()),
            CompletionChunk::RightParenthesis,
        ]);

        let offset = if cfg!(target_os="windows") && cfg!(feature="clang_3_8") {
            assert_result_eq!(results[1], EntityKind::Method, 34, None, "A", "operator=", &[
                CompletionChunk::ResultType("A &".into()),
                CompletionChunk::TypedText("operator=".into()),
                CompletionChunk::LeftParenthesis,
                CompletionChunk::Placeholder("A &&".into()),
                CompletionChunk::RightParenthesis,
            ]);
            1
        } else {
            0
        };

        assert_result_eq!(results[1 + offset], EntityKind::Destructor, 34, None, "A", "~A", &[
            CompletionChunk::ResultType("void".into()),
            CompletionChunk::TypedText("~A".into()),
            CompletionChunk::LeftParenthesis,
            CompletionChunk::RightParenthesis,
        ]);

        let brief = Some("An integer field.".into());
        assert_result_eq!(results[2 + offset], EntityKind::FieldDecl, 35, brief, "A", "a", &[
            CompletionChunk::ResultType("int".into()),
            CompletionChunk::TypedText("a".into()),
        ]);

        assert_result_eq!(results[3 + offset], EntityKind::FieldDecl, 35, None, "A", "b", &[
            CompletionChunk::ResultType("int".into()),
            CompletionChunk::TypedText("b".into()),
        ]);

        assert_result_eq!(results[4 + offset], EntityKind::FieldDecl, 35, None, "A", "c", &[
            CompletionChunk::ResultType("int".into()),
            CompletionChunk::TypedText("c".into()),
        ]);

        assert_result_eq!(results[5 + offset], EntityKind::StructDecl, 75, None, "A", "A", &[
            CompletionChunk::TypedText("A".into()),
            CompletionChunk::Text("::".into()),
        ]);
    });
}
