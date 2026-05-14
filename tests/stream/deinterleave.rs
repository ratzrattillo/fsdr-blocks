use fsdr_blocks::stream::*;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSource;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Result;
use futuresdr::runtime::Runtime;
use futuresdr::runtime::macros::connect;

#[test]
fn deinterleave_u8() -> Result<()> {
    let mut fg = Flowgraph::new();

    let deinterleaver = Deinterleave::<u8>::new();

    let orig: Vec<u8> = vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1];
    let src = VectorSource::<u8>::new(orig.clone());
    let vect_sink_0 = VectorSink::<u8>::new(1024);
    let vect_sink_1 = VectorSink::<u8>::new(1024);

    connect!(fg,
        src > deinterleaver;
        deinterleaver.out0 > vect_sink_0;
        deinterleaver.out1 > vect_sink_1;
    );
    let fg = Runtime::new().run(fg)?;

    let binding_0 = vect_sink_0.get(&fg)?;
    let snk_0 = binding_0.items();

    let binding_1 = vect_sink_1.get(&fg)?;
    let snk_1 = binding_1.items();

    assert_eq!(snk_0.len(), orig.len() / 2);
    assert_eq!(snk_0.len(), snk_1.len());
    assert!(snk_0.iter().all(|v| *v == 0));
    assert!(snk_1.iter().all(|v| *v == 1));

    Ok(())
}
