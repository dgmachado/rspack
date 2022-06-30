use std::collections::HashMap;

use hashbrown::HashSet;
use tracing::instrument;

use crate::{ModuleGraph, ModuleGraphModule};

#[derive(Debug)]
pub struct Chunk {
  pub id: String,
  pub(crate) module_uris: HashSet<String>,
  pub(crate) entry_uri: String,
  pub kind: ChunkKind,
  pub module_index: HashMap<String, usize>,
}

impl Chunk {
  pub fn new(id: String, entry_uri: String, kind: ChunkKind) -> Self {
    Self {
      id,
      module_uris: Default::default(),
      entry_uri,
      kind,
      module_index: Default::default(),
    }
  }

  pub fn calc_exec_order(&mut self, module_graph: &ModuleGraph) {
    let entries = [self.entry_uri.clone()];
    let mut visited = HashSet::new();

    let mut next_exec_order = 0;
    for entry in entries {
      let mut stack_visited: HashSet<String> = HashSet::new();
      let mut stack = vec![entry];
      while let Some(module_uri) = stack.pop() {
        if !visited.contains(&module_uri) {
          if stack_visited.contains(module_uri.as_str()) {
            self
              .module_index
              .insert(module_uri.clone(), next_exec_order);
            // tracing::debug!(
            //   "module: {:?},next_exec_order {:?}",
            //   module_uri,
            //   next_exec_order
            // );
            next_exec_order += 1;
            visited.insert(module_uri);
          } else {
            stack.push(module_uri.to_string());
            stack_visited.insert(module_uri.to_string());
            stack.append(
              &mut module_graph
                .module_by_uri(&module_uri)
                .unwrap()
                .depended_modules(module_graph)
                .into_iter()
                .rev()
                .map(|dep_mod| &dep_mod.uri)
                .cloned()
                .collect(),
            )
          }
        }
      }
    }
  }

  #[instrument]
  pub fn ordered_modules<'a>(&self, module_graph: &'a ModuleGraph) -> Vec<&'a ModuleGraphModule> {
    let mut ordered = self
      .module_uris
      .iter()
      .filter_map(|uri| module_graph.module_by_uri(uri))
      .collect::<Vec<_>>();
    ordered.sort_by_key(|m| self.module_index[&m.uri]);
    ordered
  }
}

#[derive(Debug)]
pub enum ChunkKind {
  Entry { name: String },
  Normal,
  // TODO: support it.
  // Initial,
}

impl ChunkKind {
  pub fn is_entry(&self) -> bool {
    matches!(self, ChunkKind::Entry { .. })
  }
  pub fn is_normal(&self) -> bool {
    matches!(self, ChunkKind::Normal)
  }
}