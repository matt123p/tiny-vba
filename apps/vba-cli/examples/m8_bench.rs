//! Authored feature benchmarks with independent typed assertions on every sample.
use std::{error::Error, io, time::Instant};
use vba_compiler::{CompileOptions, compile};
use vba_diagnostics::{SourceFile, SourceId};
use vba_values::{Value, VbaString};
use vba_vm::{Limits, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn Error>>;
fn ensure(condition: bool, message: &str) -> R {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message).into())
    }
}
fn main() -> R {
    let samples = std::env::var("M8_BENCH_SAMPLES")
        .ok()
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(10);
    ensure(
        (2..=200).contains(&samples),
        "benchmark samples must be 2..200",
    )?;
    println!(
        "M8 benchmark schema=1 profile=authored-language-v1 arch={} os={} samples={samples} warmup=2 clock=Instant unit=ns; Office NOT_RUN",
        std::env::consts::ARCH,
        std::env::consts::OS
    );
    for (name, source, expected) in [
        (
            "scalar-loop",
            "Function Answer() As Long\nDim i As Long,n As Long\nFor i=1 To 100\nn=n+i\nNext i\nAnswer=n\nEnd Function",
            Value::Long(5050),
        ),
        (
            "byref",
            "Function Answer() As Long\nDim n As Long\nn=21\nDoubleIt n\nAnswer=n\nEnd Function\nSub DoubleIt(ByRef n As Long)\nn=n*2\nEnd Sub",
            Value::Long(42),
        ),
        (
            "array-bounds",
            "Function Answer() As Long\nDim a(-2 To 2) As Long,i As Long,n As Long\nFor i=LBound(a) To UBound(a)\na(i)=i*i\nn=n+a(i)\nNext i\nAnswer=n\nEnd Function",
            Value::Long(10),
        ),
        (
            "utf16",
            "Function Answer() As String\nAnswer=Mid$(\"abcdef\",2,3) & CStr(Len(\"A😀\"))\nEnd Function",
            Value::String(VbaString::from_utf8("bcd3")?),
        ),
        (
            "error-handler",
            "Function Answer() As Long\nOn Error Resume Next\nDim n As Long\nn=1/0\nAnswer=Err.Number\nEnd Function",
            Value::Long(11),
        ),
        (
            "currency",
            "Function Answer() As Currency\nAnswer=CCur(1.25)*2\nEnd Function",
            Value::Currency(25000),
        ),
        (
            "date",
            "Function Answer() As Date\nAnswer=DateSerial(2024,2,29)\nEnd Function",
            Value::Date(45351.0),
        ),
        (
            "recursion",
            "Function Answer() As Long\nAnswer=Fact(6)\nEnd Function\nFunction Fact(ByVal n As Long) As Long\nIf n<2 Then\nFact=1\nElse\nFact=n*Fact(n-1)\nEnd If\nEnd Function",
            Value::Long(720),
        ),
    ] {
        let file = SourceFile::new(SourceId(1), format!("{name}.bas"), source.into())?;
        let options = CompileOptions {
            capabilities: vba_vm::capabilities(),
            ..Default::default()
        };
        let compile_start = Instant::now();
        let program = compile(&file, &options)?;
        let compile_ns = compile_start.elapsed().as_nanos();
        let mut times = Vec::new();
        let mut instructions = None;
        for sample in 0..samples + 2 {
            let start = Instant::now();
            let mut session = Session::new(
                program.clone(),
                1,
                vec![],
                None,
                Limits {
                    instructions: 100000,
                    ..Default::default()
                },
            )?;
            let mut terminal = None;
            for _ in 0..1000 {
                let outcome = session.run_slice(&mut vba_vm::NoHost, 256, &mut |_| Ok(false))?;
                if outcome != Outcome::Yielded {
                    terminal = Some(outcome);
                    break;
                }
            }
            let elapsed = start.elapsed().as_nanos();
            ensure(
                terminal == Some(Outcome::Completed(ReturnValue::Function(expected.clone()))),
                &format!("{name}: typed result mismatch: {terminal:?}"),
            )?;
            let count = session.counters().instructions;
            ensure(
                instructions.is_none_or(|previous| previous == count),
                "nonrepeatable VM instruction count",
            )?;
            instructions = Some(count);
            if sample >= 2 {
                times.push(elapsed);
            }
        }
        times.sort_unstable();
        println!(
            "BENCH {name} PASS expected={} compile_ns={compile_ns} run_min_ns={} run_median_ns={} run_max_ns={} instructions={}",
            vba_conformance::describe_value(&expected),
            times.first().ok_or("empty samples")?,
            times.get(samples / 2).ok_or("median")?,
            times.last().ok_or("empty samples")?,
            instructions.ok_or("instruction count")?
        );
    }
    Ok(())
}
