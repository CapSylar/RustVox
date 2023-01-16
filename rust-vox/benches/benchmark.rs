use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_greedy_mesher(c: &mut Criterion)
{
    // let generator = Box::new(PerlinGenerator::default());
    // let mut chunk = Chunk::new(0,0,0,generator.as_ref());

    // c.bench_function("greedy_mesher", |b| b.iter( || chunk.generate_mesh::<GreedyMesher>()));
}

fn benchmark_culling_mesher(c: &mut Criterion)
{
    // let generator = Box::new(PerlinGenerator::default());
    // let mut chunk = Chunk::new(0,0,0,generator.as_ref());

    // c.bench_function("culling_mesher", |b| b.iter( || chunk.generate_mesh::<CullingMesher>()));
}

criterion_group!(benches, benchmark_culling_mesher, benchmark_greedy_mesher);
criterion_main!(benches);