use std::{thread::scope, time::Instant};

use rayon::prelude::*;

fn main() {
    sum_million();
    prime_numbers();
    scopes();
    scopes_spawn_broadcase();
}

fn sum_million() {
    let numbers: Vec<u64> = (0..100_000_000).collect();
    let sum = numbers.par_iter().sum::<u64>();

    println!("the sum is {sum}")
}

fn prime_numbers() {
    let now = Instant::now();
    let numbers: Vec<u32> = (0..1000).collect();
    let mut primes: Vec<&u32> = numbers.par_iter().filter(|n| is_prime(**n)).collect();
    primes.par_sort_unstable();
    let elapsed = now.elapsed().as_secs_f32();
    println!("found {} primes in {} seconds", primes.len(), elapsed);
}

fn is_prime(n: u32) -> bool {
    (2..=n / 2).into_par_iter().all(|i| n % i != 0)
}

fn scopes() {
    // Explicitly sized pool
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();

    pool.spawn(|| println!("hello from a pooled thread"));

    pool.scope(|scope| {
        for n in 0..20 {
            scope.spawn(move |_| println!("hello from scoped thread {n}"));
        }
    });
    println!("hello from main theead")
}
fn scopes_spawn_broadcase() {
    // Explicitly sized pool
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();

    pool.scope(|scope| {
        scope.spawn_broadcast(|_scope, broadcast_context| {
            println!("hello from broadcast thread {}", broadcast_context.index());
        });
    });
    println!("hello from main theead")
}
