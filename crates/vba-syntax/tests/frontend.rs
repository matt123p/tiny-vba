use std::{error::Error, fs, path::Path};
use vba_diagnostics::{SourceFile, SourceId};
use vba_syntax::{ExprKind, ModuleKind, StatementKind, TokenKind, lex, parse};

fn source(text: &str) -> Result<SourceFile, vba_diagnostics::Diagnostic> {
    SourceFile::new(SourceId(41), "test.bas".into(), text.into())
}
fn function(expression: &str) -> String {
    format!("Function Answer() As Long\nAnswer = {expression}\nEnd Function\n")
}

#[test]
fn qualified_declared_types_keep_exact_unicode_crlf_spans() -> Result<(), Box<dyn Error>> {
    let file = source(
        "' λ\r\nFunction Answer(ByVal item As HostLib . Mode) As HostLib.Mode\r\nDim value As HostLib.Mode\r\nEnd Function\r\n",
    )?;
    let parsed = parse(&file, ModuleKind::Standard)?;
    assert!(parsed.syntax_valid(), "{:?}", parsed.diagnostics);
    let proc = parsed.module.procedures.first().ok_or("procedure")?;
    let ty = proc
        .parameters
        .first()
        .ok_or("parameter")?
        .ty
        .as_ref()
        .ok_or("type")?;
    assert_eq!(ty.spelling, "HostLib.Mode");
    assert_eq!(file.slice(ty.span)?, "HostLib . Mode");
    assert_eq!(
        proc.result_type.as_ref().ok_or("result")?.spelling,
        "HostLib.Mode"
    );
    Ok(())
}

#[test]
fn lossless_tokens_continuations_comments_and_crlf() -> Result<(), Box<dyn Error>> {
    let text = "' café 😀\r\nOption Explicit\r\nSub Demo()\r\nRem a comment\r\nx = 1 + _\r\n 2: y = \"a\"\"b\" ' trailing\r\nEnd Sub\r\n";
    let file = source(text)?;
    let lexed = lex(&file);
    assert!(lexed.diagnostics.is_empty(), "{:?}", lexed.diagnostics);
    let rebuilt = lexed
        .tokens
        .iter()
        .map(|t| t.text(&file))
        .collect::<Result<Vec<_>, _>>()?
        .concat();
    assert_eq!(rebuilt, text);
    assert!(
        lexed
            .tokens
            .iter()
            .any(|t| t.kind == TokenKind::Continuation)
    );
    let parsed = parse(&file, ModuleKind::Standard)?;
    assert!(parsed.syntax_valid(), "{:?}", parsed.diagnostics);
    let procedure = parsed
        .module
        .procedures
        .first()
        .ok_or("missing procedure")?;
    let a = procedure.body.first().ok_or("missing assignment")?;
    let b = procedure.body.get(1).ok_or("missing second assignment")?;
    assert_ne!(a.site.id, b.site.id);
    assert_eq!(a.site.id.source, SourceId(41));
    assert_eq!(file.slice(a.site.span)?, "x = 1 + _\r\n 2");
    let StatementKind::Assign { value, .. } = &b.kind else {
        return Err("expected assignment".into());
    };
    assert!(matches!(&value.kind, ExprKind::String(s) if s == "a\"b"));
    Ok(())
}
#[test]
fn examples_and_twenty_scenario_bodies_parse_without_execution() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for relative in [
        "examples/pure-vba/Arithmetic.bas",
        "examples/foreign-host/FillCells.bas",
        "tests/syntax/m1/control.bas",
        "tests/syntax/m1/metadata.cls",
    ] {
        let file = source(&fs::read_to_string(root.join(relative))?)?;
        let kind = if relative.ends_with(".cls") {
            ModuleKind::Class
        } else {
            ModuleKind::Standard
        };
        let parsed = parse(&file, kind)?;
        assert!(
            parsed.syntax_valid(),
            "{relative}: {:?}",
            parsed.diagnostics
        );
        assert!(!parsed.module.procedures.is_empty());
    }
    let mut count = 0;
    for entry in fs::read_dir(root.join("tests/fixtures/m0"))? {
        let text = fs::read_to_string(entry?.path())?;
        let (_, body) = text.split_once("---\n").ok_or("missing body")?;
        let parsed = parse(&source(body)?, ModuleKind::Standard)?;
        assert!(parsed.syntax_valid(), "{:?}", parsed.diagnostics);
        count += 1;
    }
    assert_eq!(count, 20);
    Ok(())
}
#[test]
fn operator_precedence_and_parentheses_are_structural() -> Result<(), Box<dyn Error>> {
    let parsed = parse(
        &source(&function("Not 1 + 2 * 3 = -2 ^ 2 And True"))?,
        ModuleKind::Standard,
    )?;
    assert!(parsed.syntax_valid(), "{:?}", parsed.diagnostics);
    let statement = parsed
        .module
        .procedures
        .first()
        .and_then(|p| p.body.first())
        .ok_or("missing statement")?;
    let StatementKind::Assign { value, .. } = &statement.kind else {
        return Err("assignment".into());
    };
    let ExprKind::Binary { op, left, .. } = &value.kind else {
        return Err("And".into());
    };
    assert_eq!(op, "and");
    let ExprKind::Unary { op, value } = &left.kind else {
        return Err("Not".into());
    };
    assert_eq!(op, "not");
    let ExprKind::Binary { op, left, right } = &value.kind else {
        return Err("comparison".into());
    };
    assert_eq!(op, "=");
    assert!(
        matches!(&left.kind, ExprKind::Binary { op, right, .. } if op == "+" && matches!(&right.kind, ExprKind::Binary { op, .. } if op == "*"))
    );
    assert!(
        matches!(&right.kind, ExprKind::Unary { op, value } if op == "-" && matches!(&value.kind, ExprKind::Binary { op, .. } if op == "^"))
    );
    Ok(())
}
#[test]
fn reserved_word_argument_labels_preserve_spelling_and_spans() -> Result<(), Box<dyn Error>> {
    let file = source("Sub Demo()\nclient.Run End:=3, Type:=4\nEnd Sub\n")?;
    let parsed = parse(&file, ModuleKind::Standard)?;
    assert!(parsed.syntax_valid(), "{:?}", parsed.diagnostics);
    let StatementKind::Call { arguments, .. } = &parsed
        .module
        .procedures
        .first()
        .ok_or("procedure")?
        .body
        .first()
        .ok_or("call")?
        .kind
    else {
        return Err("call".into());
    };
    for (argument, expected) in arguments.iter().zip(["End", "Type"]) {
        let label = argument.name.as_ref().ok_or("named argument")?;
        assert_eq!(label.spelling, expected);
        assert_eq!(file.slice(label.span)?, expected);
    }
    assert_eq!(arguments.len(), 2);
    let invalid = source("Sub Demo()\nDim End As Long\nEnd Sub\n")?;
    assert!(
        !parse(&invalid, ModuleKind::Standard)?.syntax_valid(),
        "ordinary declarations still reject reserved identifiers"
    );
    Ok(())
}

#[test]
fn call_forms_named_omitted_and_parenthesized_arguments_stay_distinct() -> Result<(), Box<dyn Error>>
{
    let file = source(
        "Sub Demo()\nFoo x\nFoo (x)\nCall Foo(x)\nx = Foo(, right:=2)\nfactory(1).Run x\nEnd Sub\n",
    )?;
    let parsed = parse(&file, ModuleKind::Standard)?;
    assert!(parsed.syntax_valid(), "{:?}", parsed.diagnostics);
    let body = &parsed.module.procedures.first().ok_or("procedure")?.body;
    let StatementKind::Call {
        arguments,
        explicit,
        ..
    } = &body.first().ok_or("call")?.kind
    else {
        return Err("call".into());
    };
    assert!(!explicit);
    assert!(matches!(
        arguments
            .first()
            .and_then(|a| a.value.as_ref())
            .map(|v| &v.kind),
        Some(ExprKind::Name(_))
    ));
    let StatementKind::Call { arguments, .. } = &body.get(1).ok_or("call")?.kind else {
        return Err("call".into());
    };
    assert!(matches!(
        arguments
            .first()
            .and_then(|a| a.value.as_ref())
            .map(|v| &v.kind),
        Some(ExprKind::Parenthesized(_))
    ));
    assert!(matches!(
        &body.get(2).ok_or("call")?.kind,
        StatementKind::Call { explicit: true, .. }
    ));
    let StatementKind::Assign { value, .. } = &body.get(3).ok_or("assign")?.kind else {
        return Err("assign".into());
    };
    let ExprKind::Apply { arguments, .. } = &value.kind else {
        return Err("apply".into());
    };
    assert!(arguments.first().is_some_and(|a| a.value.is_none()));
    assert_eq!(
        arguments
            .get(1)
            .and_then(|a| a.name.as_ref())
            .map(|n| n.spelling.as_str()),
        Some("right")
    );
    Ok(())
}
#[test]
fn exact_error_span_and_bounded_recovery_preserve_following_statements()
-> Result<(), Box<dyn Error>> {
    let text = include_str!("../../../tests/syntax/m1/invalid.bas");
    let file = source(text)?;
    let parsed = parse(&file, ModuleKind::Standard)?;
    assert!(!parsed.syntax_valid());
    assert_eq!(parsed.diagnostics.len(), 1);
    let diagnostic = parsed.diagnostics.first().ok_or("diagnostic")?;
    assert_eq!(diagnostic.code, "VBA0101");
    let span = diagnostic.span.ok_or("span")?;
    assert_eq!(file.slice(span)?, ")");
    assert_eq!(file.position(span)?, (3, 19));
    assert_eq!(
        parsed
            .module
            .procedures
            .first()
            .ok_or("recovered procedure")?
            .body
            .len(),
        2
    );
    Ok(())
}
#[test]
fn same_line_ids_and_metadata_retain_spelling_and_source_order() -> Result<(), Box<dyn Error>> {
    let file = source(
        "Attribute VB_Name = \"Demo\"\nOption Explicit\nsUb Mixed()\nx=1: y=2: z=x+y\nEnd Sub\n",
    )?;
    let parsed = parse(&file, ModuleKind::Standard)?;
    assert!(parsed.syntax_valid(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.module.attributes.len(), 1);
    let procedure = parsed.module.procedures.first().ok_or("procedure")?;
    assert_eq!(procedure.name.spelling, "Mixed");
    assert_eq!(procedure.name.key(), "mixed");
    let mut ordinals = Vec::new();
    for statement in &procedure.body {
        assert_eq!(file.position(statement.site.span)?.0, 4);
        ordinals.push(statement.site.id.ordinal);
    }
    assert_eq!(ordinals, [0, 1, 2]);
    assert_eq!(procedure.end.id.ordinal, 3);
    Ok(())
}
#[test]
fn malformed_and_unsupported_syntax_never_reports_success() -> Result<(), Box<dyn Error>> {
    for body in [
        "x =",
        "x = (1",
        "If True Then\nx=1",
        "For i = 1 To 2\nx=1",
        "Call Foo 1",
        "Foo(1, 2)",
        "Set x",
        "x=Foo(a:=)",
        "Dim x(2 To) As Long",
        "On Error Resume Missing",
        "Select Case x",
        "For Each x In y",
        "GoTo",
        "While True",
        "x = #1/1/2000#",
        "[escaped] = 1",
        "x = \"broken",
        "x = 1e+",
        "x = &H",
        "_\nx=1",
        "x = 1 _ ' comment",
    ] {
        let text = format!("Sub Demo()\n{body}\nEnd Sub\n");
        let parsed = parse(&source(&text)?, ModuleKind::Standard)?;
        assert!(!parsed.syntax_valid(), "accepted {body:?}");
        assert!(parsed.diagnostics.len() <= vba_syntax::MAX_DIAGNOSTICS);
        assert!(parsed.diagnostics.iter().all(|d| d.span.is_some()));
    }
    Ok(())
}
#[test]
fn nesting_tokens_and_recovery_have_hard_limits() -> Result<(), Box<dyn Error>> {
    for expression in [
        format!("{}1{}", "(".repeat(2000), ")".repeat(2000)),
        format!("{}1", "Not ".repeat(2000)),
        (0..2000).map(|_| "1").collect::<Vec<_>>().join(" + "),
        (0..2000).map(|_| "x").collect::<Vec<_>>().join("."),
    ] {
        let parsed = parse(&source(&function(&expression))?, ModuleKind::Standard)?;
        assert!(!parsed.syntax_valid());
        assert!(parsed.diagnostics.iter().any(|d| d.code == "VBA0102"));
    }
    let blocks = format!(
        "Sub Demo()\n{}x=1\n{}End Sub\n",
        "If True Then\n".repeat(2000),
        "End If\n".repeat(2000)
    );
    let parsed = parse(&source(&blocks)?, ModuleKind::Standard)?;
    assert!(!parsed.syntax_valid());
    assert!(parsed.diagnostics.iter().any(|d| d.code == "VBA0102"));
    let lexed = lex(&source(&"x ".repeat(40_000))?);
    assert!(lexed.tokens.len() <= vba_syntax::MAX_TOKENS);
    assert!(lexed.diagnostics.iter().any(|d| d.code == "VBA0102"));
    let parsed = parse(
        &source(&format!("Sub Demo()\n{}End Sub\n", "x=)\n".repeat(1000)))?,
        ModuleKind::Standard,
    )?;
    assert_eq!(parsed.diagnostics.len(), vba_syntax::MAX_DIAGNOSTICS);
    Ok(())
}

#[test]
fn token_literals_and_newline_variants_preserve_exact_spelling() -> Result<(), Box<dyn Error>> {
    for newline in ["\n", "\r\n", "\r"] {
        let text = format!(
            "Sub Demo(){newline}x = &H2A&: y = &O52: z = .5e+2: s$ = \"é😀\"{newline}End Sub{newline}"
        );
        let file = source(&text)?;
        let parsed = parse(&file, ModuleKind::Standard)?;
        assert!(parsed.syntax_valid(), "{:?}", parsed.diagnostics);
        let body = &parsed.module.procedures.first().ok_or("procedure")?.body;
        let StatementKind::Assign { value, .. } = &body.first().ok_or("assignment")?.kind else {
            return Err("assignment".into());
        };
        assert!(matches!(&value.kind, ExprKind::Number(raw) if raw == "&H2A&"));
        for statement in body {
            assert_eq!(file.position(statement.site.span)?.0, 2);
        }
    }
    Ok(())
}

#[test]
fn truncated_and_generated_inputs_return_bounded_diagnostics() -> Result<(), Box<dyn Error>> {
    let specimen = include_str!("../../../tests/syntax/m1/control.bas");
    for (end, _) in specimen.char_indices().step_by(7) {
        let text = specimen.get(..end).ok_or("boundary")?;
        let result = parse(&source(text)?, ModuleKind::Standard)?;
        assert!(result.diagnostics.len() <= vba_syntax::MAX_DIAGNOSTICS);
    }
    // Deterministic malformed Unicode/token mixtures; this is a regression stress check, not fuzzing.
    let alphabet = [
        'x', ' ', '=', '(', ')', ':', ',', '\r', '\n', '"', '\'', '_', '+', '#', 'é', '😀', '\0',
    ];
    let mut seed = 17usize;
    for _ in 0..1000 {
        let mut text = String::new();
        for _ in 0..80 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            text.push(*alphabet.get(seed % alphabet.len()).ok_or("alphabet")?);
        }
        let result = parse(&source(&text)?, ModuleKind::Standard)?;
        assert!(result.diagnostics.len() <= vba_syntax::MAX_DIAGNOSTICS);
    }
    Ok(())
}

#[test]
fn nested_control_sites_have_unique_ordered_ids_and_exact_spans() -> Result<(), Box<dyn Error>> {
    let file = source(
        "Sub Demo()\nFor i = 1 To 2\nIf i = 1 Then\nx=1: y=2\nElseIf i = 2 Then\nx=3\nElse\nx=4\nEnd If\nNext i\nEnd Sub\n",
    )?;
    let parsed = parse(&file, ModuleKind::Standard)?;
    assert!(parsed.syntax_valid(), "{:?}", parsed.diagnostics);
    let procedure = parsed.module.procedures.first().ok_or("procedure")?;
    let outer = procedure.body.first().ok_or("For")?;
    let StatementKind::For(for_loop) = &outer.kind else {
        return Err("For".into());
    };
    let conditional = for_loop.body.first().ok_or("If")?;
    let StatementKind::If {
        arms,
        otherwise,
        end,
        ..
    } = &conditional.kind
    else {
        return Err("If".into());
    };
    let first = arms.first().ok_or("first arm")?;
    let second = arms.get(1).ok_or("second arm")?;
    assert_eq!(first.site.id, conditional.site.id);
    let sites = [
        outer.site,
        first.site,
        first.body.first().ok_or("x")?.site,
        first.body.get(1).ok_or("y")?.site,
        second.site,
        second.body.first().ok_or("x")?.site,
        otherwise.first().ok_or("else")?.site,
        end.ok_or("End If")?,
        for_loop.next,
        procedure.end,
    ];
    let expected = [
        "For i = 1 To 2",
        "If i = 1 Then",
        "x=1",
        "y=2",
        "ElseIf i = 2 Then",
        "x=3",
        "x=4",
        "End If",
        "Next i",
        "End Sub",
    ];
    for (index, (site, text)) in sites.iter().zip(expected).enumerate() {
        assert_eq!(site.id.ordinal as usize, index);
        assert_eq!(file.slice(site.span)?, text);
    }
    Ok(())
}

#[test]
fn control_nesting_and_all_identifier_positions_are_bounded() -> Result<(), Box<dyn Error>> {
    for body in [
        format!(
            "{}x=1\n{}",
            "If True Then\n".repeat(1000),
            "End If\n".repeat(1000)
        ),
        format!(
            "{}x=1\n{}",
            "For i=1 To 2\n".repeat(1000),
            "Next i\n".repeat(1000)
        ),
        format!("{} = 1", "x".repeat(129)),
        format!("obj.{} = 1", "x".repeat(129)),
        format!("Call obj.{}()", "x".repeat(129)),
    ] {
        let parsed = parse(
            &source(&format!("Sub Demo()\n{body}\nEnd Sub\n"))?,
            ModuleKind::Standard,
        )?;
        assert!(!parsed.syntax_valid());
        assert!(parsed.diagnostics.iter().any(|d| d.code == "VBA0102"));
        assert!(parsed.diagnostics.len() <= vba_syntax::MAX_DIAGNOSTICS);
    }
    Ok(())
}

#[test]
fn control_headers_terminators_and_parameter_metadata_are_preserved() -> Result<(), Box<dyn Error>>
{
    let file = source(include_str!("../../../tests/syntax/m1/control.bas"))?;
    let parsed = parse(&file, ModuleKind::Standard)?;
    assert!(parsed.syntax_valid(), "{:?}", parsed.diagnostics);
    let procedure = parsed.module.procedures.first().ok_or("procedure")?;
    assert!(file.slice(procedure.span)?.starts_with("Public Function"));
    let optional = procedure.parameters.get(1).ok_or("optional parameter")?;
    assert!(optional.optional && optional.by_val && optional.default.is_some());
    let statement = procedure.body.get(2).ok_or("For statement")?;
    let StatementKind::For(loop_) = &statement.kind else {
        return Err("For".into());
    };
    assert_eq!(
        file.slice(statement.site.span)?,
        "For i = 1 To Limit Step amount"
    );
    assert_eq!(file.slice(loop_.next.span)?, "Next i");
    assert!(loop_.next.id.ordinal > statement.site.id.ordinal);
    let StatementKind::If { arms, end, .. } = &loop_.body.first().ok_or("If")?.kind else {
        return Err("If".into());
    };
    assert_eq!(arms.len(), 2);
    let elseif = arms.get(1).ok_or("ElseIf")?;
    assert_eq!(file.slice(elseif.site.span)?, "ElseIf i = 1 Then");
    assert_eq!(file.slice(end.ok_or("End If")?.span)?, "End If");
    Ok(())
}
