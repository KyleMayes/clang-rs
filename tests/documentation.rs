use clang::*;
use clang::documentation::*;

pub fn test(clang: &Clang) {
    let source = r#"
        int a();
        /// \brief This is a function.
        ///
        /// This function does stuff and then returns an \c int for reasons unknown.
        ///
        /// <br />
        /// <a href="http://example.com">More information.</a>
        ///
        /// \tparam T This template parameter doesn't actually do anything.
        /// \param [in] i This parameter alters the behavior of the function in some way.
        ///
        /// \verbatim *nullptr \endverbatim
        template <typename T>
        int b(int i) { return i; }
    "#;

    super::with_entity(&clang, source, |e| {
        let children = e.get_children();
        assert_eq!(children.len(), 2);

        assert!(children[0].get_parsed_comment().is_none());

        let comment = children[1].get_parsed_comment().unwrap();
        assert!(!comment.as_html().is_empty());
        assert!(!comment.as_xml().is_empty());

        let children = comment.get_children();
        assert_eq!(children.len(), 9);

        assert_eq!(children[0], CommentChild::Paragraph(vec![
            CommentChild::Text(" ".into()),
        ]));
        assert_eq!(children[1], CommentChild::BlockCommand(BlockCommand {
            command: "brief".into(), arguments: vec![], children: vec![
                CommentChild::Text(" This is a function.".into()),
            ]
        }));
        assert_eq!(children[2], CommentChild::Paragraph(vec![
            CommentChild::Text(" This function does stuff and then returns an ".into()),
            CommentChild::InlineCommand(InlineCommand {
                command: "c".into(),
                arguments: vec!["int".into()],
                style: Some(InlineCommandStyle::Monospace),
            }),
            CommentChild::Text(" for reasons unknown.".into()),
        ]));
        assert_eq!(children[3], CommentChild::Paragraph(vec![
            CommentChild::Text(" ".into()),
            CommentChild::HtmlStartTag(HtmlStartTag {
                name: "br".into(), attributes: vec![], closing: true
            }),
            CommentChild::Text(" ".into()),
            CommentChild::HtmlStartTag(HtmlStartTag {
                name: "a".into(), attributes: vec![
                    ("href".into(), "http://example.com".into())
                ], closing: false
            }),
            CommentChild::Text("More information.".into()),
            CommentChild::HtmlEndTag("a".into()),
        ]));
        assert_eq!(children[4], CommentChild::Paragraph(vec![
            CommentChild::Text(" ".into()),
        ]));
        assert_eq!(children[5], CommentChild::TParamCommand(TParamCommand {
            position: Some((1, 0)), parameter: "T".into(), children: vec![
                CommentChild::Text(" This template parameter doesn't actually do anything.".into()),
                CommentChild::Text(" ".into()),
            ]
        }));
        assert_eq!(children[6], CommentChild::ParamCommand(ParamCommand {
            index: Some(0), parameter: "i".into(), direction: Some(ParameterDirection::In), children: vec![
                CommentChild::Text(" This parameter alters the behavior of the function in some way.".into()),
            ]
        }));
        assert_eq!(children[7], CommentChild::Paragraph(vec![
            CommentChild::Text(" ".into()),
        ]));
        assert_eq!(children[8], CommentChild::VerbatimCommand(vec![" *nullptr ".into()]));
    });
}
