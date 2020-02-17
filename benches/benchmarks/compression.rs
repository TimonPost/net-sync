use criterion::{criterion_group, Criterion};
use net_sync::compression::{lz4::Lz4, CompressionStrategy, ModificationCompressor};
use track::{preclude::Uuid, ModificationEvent};

struct Postion;

fn compress<T: CompressionStrategy>(
    compressor: &ModificationCompressor<T>,
    packet: ModificationEvent,
) {
    compressor.compress(&packet.modified_fields);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Compression with lz4", |b| {
        let compressor = ModificationCompressor::new(Lz4);
        let packet = ModificationEvent::new(vec![19; 5000], Some(Uuid::new_v4()));

        b.iter(|| compress::<Lz4>(&compressor, packet.clone()));
    });
}

criterion_group!(compression, criterion_benchmark);
