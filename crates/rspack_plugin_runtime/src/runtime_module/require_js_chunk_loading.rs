use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RUNTIME_MODULE_STAGE_ATTACH,
};
use rspack_identifier::Identifier;

use super::utils::{chunk_has_js, get_output_dir};
use crate::impl_runtime_module;
use crate::runtime_module::utils::{get_initial_chunk_ids, stringify_chunks};

#[derive(Debug, Default, Eq)]
pub struct RequireChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  runtime_requirements: RuntimeGlobals,
}

impl RequireChunkLoadingRuntimeModule {
  pub fn new(runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/require_chunk_loading"),
      chunk: None,
      runtime_requirements,
    }
  }
}

impl RuntimeModule for RequireChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let chunk = compilation
      .chunk_by_ukey
      .get(&self.chunk.expect("The chunk should be attached."))
      .expect("Chunk is not found, make sure you had attach chunkUkey successfully.");
    let with_hmr = self
      .runtime_requirements
      .contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_external_install_chunk = self
      .runtime_requirements
      .contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);
    let root_output_dir = get_output_dir(chunk, compilation, true);
    let mut source = ConcatSource::default();

    if self.runtime_requirements.contains(RuntimeGlobals::BASE_URI) {
      source.add(RawSource::from(format!(
        "{} = require(\"url\").pathToFileURL({});\n",
        RuntimeGlobals::BASE_URI,
        if &root_output_dir != "./" {
          format!("__dirname + \"/{}\"", root_output_dir)
        } else {
          "__filename".to_string()
        }
      )))
    }

    if with_hmr {
      let state_expression = format!("{}_require", RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX);
      source.add(RawSource::from(format!(
        "var installedChunks = {} = {} || {};\n",
        state_expression,
        state_expression,
        &stringify_chunks(&initial_chunks, 1)
      )));
    } else {
      source.add(RawSource::from(format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunks, 0)
      )));
    }

    let with_loading = self
      .runtime_requirements
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);

    if with_loading || with_external_install_chunk {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading.js"
      )));
    }

    if with_loading {
      source.add(RawSource::from(
        include_str!("runtime/require_chunk_loading_with_loading.js")
          // TODO
          .replace("JS_MATCHER", "chunkId")
          .replace("$OUTPUT_DIR$", &root_output_dir),
      ));
    }

    if with_hmr {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading_with_hmr.js"
      )));
      source.add(RawSource::from(
        include_str!("runtime/javascript_hot_module_replacement.js").replace("$key$", "jsonp"),
      ));
    }

    if self
      .runtime_requirements
      .contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST)
    {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading_with_hmr_manifest.js"
      )));
    }

    if self
      .runtime_requirements
      .contains(RuntimeGlobals::ON_CHUNKS_LOADED)
    {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading_with_on_chunk_load.js"
      )));
    }

    if with_external_install_chunk {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading_with_external_install_chunk.js"
      )));
    }

    source.boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> u8 {
    RUNTIME_MODULE_STAGE_ATTACH
  }
}

impl_runtime_module!(RequireChunkLoadingRuntimeModule);
