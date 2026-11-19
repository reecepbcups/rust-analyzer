//! Applies changes to the IDE state transactionally.

use std::time::{Duration, Instant};

use profile::Bytes;
use salsa::Database as _;

use crate::{ChangeWithProcMacros, RootDatabase};

impl RootDatabase {
    pub fn apply_change(&mut self, change: ChangeWithProcMacros) -> Duration {
        let _p = tracing::info_span!("RootDatabase::apply_change").entered();
        let now = Instant::now();
        self.trigger_cancellation();
        let elapsed = now.elapsed();
        tracing::trace!("apply_change {:?}", change);
        change.apply(self);
        elapsed
    }

    // Feature: Memory Usage
    //
    // Clears rust-analyzer's internal database and prints memory usage statistics.
    //
    // | Editor  | Action Name |
    // |---------|-------------|
    // | VS Code | **rust-analyzer: Memory Usage (Clears Database)**

    // ![Memory Usage](https://user-images.githubusercontent.com/48062697/113065592-08559f00-91b1-11eb-8c96-64b88068ec02.gif)
    pub fn per_query_memory_usage(&mut self) -> Vec<(String, Bytes, usize)> {
        let info = (self as &mut dyn salsa::Database).memory_usage();
        let size = |it: &salsa::IngredientInfo| {
            let heap = it.heap_size_of_fields().unwrap_or(0);
            Bytes::new((it.size_of_fields() + it.size_of_metadata() + heap) as isize)
        };
        let mut acc: Vec<(String, Bytes, usize)> = info
            .structs
            .iter()
            .map(|it| (it.debug_name().to_owned(), size(it), it.count()))
            .chain(info.queries.iter().map(|(name, it)| ((*name).to_owned(), size(it), it.count())))
            .collect();

        acc.sort_by_key(|it| std::cmp::Reverse(it.1));
        acc
    }
}
