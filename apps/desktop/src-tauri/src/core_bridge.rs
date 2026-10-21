use std::sync::{Arc, Mutex};
use tauri::Manager;
use toolbox_core::{
    contracts::{Request, Response},
    paths::DataPaths,
    permissions::HostCaller,
    service::CoreService,
    AppError,
};

#[derive(Default)]
pub struct CoreState(Arc<Mutex<Option<CoreService>>>);

#[tauri::command]
pub async fn core_dispatch(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    request: serde_json::Value,
) -> Response {
    // Tauri ACL is the first layer; derive identity from the actual invoking
    // WebView as a second layer. caller_context is never trusted as identity.
    let trusted = window.label() == "main"
        && window.url().is_ok_and(|u| {
            u.username().is_empty()
                && u.password().is_none()
                && ((u.scheme() == "tauri"
                    && u.host_str() == Some("localhost")
                    && u.port().is_none())
                    || (u.scheme() == "http"
                        && u.host_str() == Some("tauri.localhost")
                        && u.port().is_none())
                    || (cfg!(debug_assertions)
                        && u.scheme() == "http"
                        && u.host_str() == Some("127.0.0.1")
                        && u.port() == Some(1420)))
        });
    if !trusted {
        return Response::from_result("invalid".into(), Err(AppError::new("ACCESS_DENIED")));
    }
    if request.to_string().len() > 65536 {
        return Response::from_result("invalid".into(), Err(AppError::new("INVALID_REQUEST")));
    }
    let request: Request = match serde_json::from_value(request) {
        Ok(r) => r,
        Err(_) => {
            return Response::from_result("invalid".into(), Err(AppError::new("INVALID_REQUEST")))
        }
    };
    if let Err(e) = request.validate() {
        return Response::from_result("invalid".into(), Err(e));
    }
    let id = request.request_id.clone();
    let fallback_id = id.clone();
    // Clone owned state before awaiting so no borrowed CommandArg crosses the
    // async command boundary; the public response envelope remains unchanged.
    let core = Arc::clone(&app.state::<CoreState>().0);
    // Opening/migrating/SQLite locking never blocks the UI thread; lazy first
    // use means no optional-domain work before the first rendered frame.
    tauri::async_runtime::spawn_blocking(move || {
        let execute = || -> Result<Response,AppError> {
            let mut service=core.lock().map_err(|_| AppError::new("STORAGE_FAILED"))?;
            if service.is_none() {
                let exe=std::env::current_exe().map_err(|_|AppError::new("STORAGE_FAILED"))?;
                let app_data=app.path().app_local_data_dir().map_err(|_|AppError::new("STORAGE_FAILED"))?;
                #[cfg(feature="core-validation")]
                let paths=if let Some(root)=std::env::var_os("TOOLBOX_CORE_TEST_ROOT") { DataPaths::at_root(std::path::Path::new(&root))? } else { DataPaths::resolve(&exe,&app_data)? };
                #[cfg(not(feature="core-validation"))]
                let paths=DataPaths::resolve(&exe,&app_data)?;
                *service=Some(CoreService::open(&paths)?);
            }
            let response=service.as_ref().ok_or_else(||AppError::new("STORAGE_FAILED"))?.dispatch(HostCaller::LocalShell,request);
            #[cfg(feature="core-validation")]
            if id=="wf03-ack" {
                if let Some(path)=std::env::var_os("TOOLBOX_CORE_REPORT") {
                    let value=serde_json::to_value(&response).map_err(|_|AppError::new("INVALID_DATA"))?;
                    if value["success"]==true && value["data"]["third_party_execution"]==false {
                        std::fs::write(path,serde_json::json!({"result":"PASS","scope":"real JS -> core_dispatch -> SQLite -> JS -> acknowledgement","response":value}).to_string()).map_err(|_|AppError::diagnostic_failed())?;
                    }
                }
            }
            Ok(response)
        };
        execute().unwrap_or_else(|e| Response::from_result(id,Err(e)))
    }).await.unwrap_or_else(|_|Response::from_result(fallback_id,Err(AppError::new("STORAGE_FAILED"))))
}

#[cfg(feature = "core-validation")]
pub fn maybe_probe(window: &tauri::WebviewWindow) -> Result<(), AppError> {
    if std::env::var_os("TOOLBOX_CORE_REPORT").is_some() {
        window.eval(r#"(async()=>{
          const invoke=window.__TAURI_INTERNALS__.invoke;
          const request=(id,command)=>({request_id:id,api_version:1,caller_context:{module_id:'toolbox.shell'},params:{command,args:{}}});
          const result=await invoke('core_dispatch',{request:request('wf03-list','modules.list')});
          if(result.success!==true || result.api_version!==1 || result.data.length!==7 || !result.data.every(m=>m.kind==='builtin')) throw Error('Invalid native module response');
          await invoke('core_dispatch',{request:request('wf03-ack','modules.capabilities')});
        })().catch(()=>{});"#).map_err(|_|AppError::diagnostic_failed())?;
    }
    Ok(())
}
