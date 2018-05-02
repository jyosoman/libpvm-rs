use ingest::{ingest_stream,
             pvm::PVM};
use iostream::IOStream;
use neo4j_glue::{CSVView, Neo4JView};
use query::low::count_processes;
use std::{collections::HashMap, sync::mpsc};

use cfg::Config;
use views::{View, ViewInst, ViewCoordinator};

use neo4j::Neo4jDB;

type EngineResult<T> = Result<T, &'static str>;

pub struct Pipeline {
    pvm: PVM,
    view_ctrl: ViewCoordinator,
}

pub struct Engine {
    cfg: Config,
    pipeline: Option<Pipeline>,
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.shutdown_pipeline().ok();
    }
}

impl Engine {
    pub fn new(cfg: Config) -> Engine {
        Engine {
            cfg,
            pipeline: None,
        }
    }

    pub fn init_pipeline(&mut self) -> EngineResult<()> {
        if self.pipeline.is_some() {
            return Err("Pipeline already running");
        }
        let (send, recv) = mpsc::sync_channel(100_000);
        let mut view_ctrl = ViewCoordinator::new(recv);
        let neo4j_view_id = view_ctrl.register_view_type::<Neo4JView>();
        if !self.cfg.suppress_default_views {
            view_ctrl.create_view_inst(neo4j_view_id, hashmap!(), &self.cfg);
        }
        view_ctrl.register_view_type::<CSVView>();
        self.pipeline = Some(Pipeline {
            pvm: PVM::new(send),
            view_ctrl,
        });
        Ok(())
    }

    pub fn shutdown_pipeline(&mut self) -> EngineResult<()> {
        if let Some(pipeline) = self.pipeline.take() {
            pipeline.pvm.shutdown();
            pipeline.view_ctrl.shutdown();
            Ok(())
        } else {
            Err("Pipeline not running")
        }
    }

    pub fn print_cfg(&self) {
        println!("libPVM Config: {:?}", self.cfg);
    }

    pub fn list_view_types(&self) -> Result<Vec<&View>, &'static str> {
        if let Some(ref pipeline) = self.pipeline {
            Ok(pipeline.view_ctrl.list_view_types())
        } else {
            Err("Pipeline not running")
        }
    }

    pub fn create_view_by_id(
        &mut self,
        view_id: usize,
        params: HashMap<String, String>,
    ) -> Result<usize, &'static str> {
        if let Some(ref mut pipeline) = self.pipeline {
            Ok(pipeline
                .view_ctrl
                .create_view_inst(view_id, params, &self.cfg))
        } else {
            Err("Pipeline not running")
        }
    }

    pub fn list_running_views(&self) -> Result<Vec<&ViewInst>, &'static str> {
        if let Some(ref pipeline) = self.pipeline {
            Ok(pipeline.view_ctrl.list_view_insts())
        } else {
            Err("Pipeline not running")
        }
    }

    pub fn ingest_stream(&mut self, stream: IOStream) -> Result<(), &'static str> {
        if let Some(ref mut pipeline) = self.pipeline {
            Ok(ingest_stream(stream, &mut pipeline.pvm))
        } else {
            Err("Pipeline not running")
        }
    }

    pub fn count_processes(&self) -> i64 {
        let mut db = Neo4jDB::connect(
            &self.cfg.db_server,
            &self.cfg.db_user,
            &self.cfg.db_password,
        ).unwrap();
        count_processes(&mut db)
    }
}
