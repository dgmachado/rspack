use rspack_core::Plugin;
use rspack_error::Result;

use crate::id_helpers::{
  assign_ascending_module_ids, assign_names_par, compare_modules_by_identifier,
  get_long_module_name, get_short_module_name, get_used_module_ids_and_modules,
};

#[derive(Debug, Default)]
pub struct NamedModuleIdsPlugin {}

impl Plugin for NamedModuleIdsPlugin {
  fn name(&self) -> &'static str {
    "NamedModuleIdsPlugin"
  }

  fn module_ids(&mut self, compilation: &mut rspack_core::Compilation) -> Result<()> {
    // Align with https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/ids/NamedModuleIdsPlugin.js
    let context: &str = &compilation.options.context.to_string_lossy();
    let (mut used_ids, modules) = get_used_module_ids_and_modules(compilation, None);
    let modules = modules
      .into_iter()
      .filter_map(|i| compilation.module_graph.module_by_identifier(&i))
      .collect::<Vec<_>>();
    let chunk_graph = &mut compilation.chunk_graph;

    let unnamed_modules = assign_names_par(
      modules,
      |m| get_short_module_name(m, context),
      |module, short_name| get_long_module_name(short_name, module, context),
      |a, b| compare_modules_by_identifier(a, b),
      &mut used_ids,
      |m, name| chunk_graph.set_module_id(m.identifier(), name),
    );

    if !unnamed_modules.is_empty() {
      assign_ascending_module_ids(&used_ids, unnamed_modules, chunk_graph)
    }
    Ok(())
  }
}
