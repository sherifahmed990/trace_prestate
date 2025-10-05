use reqwest::Client;
use revm::context::BlockEnv;
use revm::primitives::U256;
use serde_json::json;
use trace_prestate::{block::{create_block_env_from_block_details, GetBlockByNumberResponse}, database::PrestateTracerResponse, json_rpc::JsonRpcResponse, trace::{op_trace_transaction, trace_transaction}};
use std::env;

use std::str::FromStr;


use revm::primitives::Bytes;

async fn get_block_by_number(
    rpc_url: &str
) -> Result<GetBlockByNumberResponse, Box<dyn std::error::Error>> {
    // JSON-RPC request payload
    let request_body = json!({
        "jsonrpc":"2.0",
        "method":"eth_getBlockByNumber",
        "params":["latest", false],
        "id":1
    });
    // Create HTTP client
    let client = Client::new();

    // Send request
    let res = client
        .post(rpc_url) // replace with your RPC endpoint
        .json(&request_body)
        .send()
        .await?;

    // Parse response
    let response: GetBlockByNumberResponse = res.json().await?;
    Ok(response)
}

async fn get_prestate_trace(
    rpc_url: &str, from: &str, to: &str, data: &str, block_number_hex: String
) -> Result<PrestateTracerResponse, Box<dyn std::error::Error>> {
    // JSON-RPC request payload
    let request_body = json!({
        "jsonrpc":"2.0",
        "method":"debug_traceCall",
        "params":[
            {
                "from": from,
                "to": to,
                "data": data
            },
            block_number_hex,
            {
              "tracer": "prestateTracer"
            }
        ],
        "id":1
    });
    // Create HTTP client
    let client = Client::new();

    // Send request
    let res = client
        .post(rpc_url) // replace with your RPC endpoint
        .json(&request_body)
        .send()
        .await?;

    // Parse response
    let response: PrestateTracerResponse = res.json().await?;
    Ok(response)
}

async fn get_nonce(
    rpc_url: &str, address: &str
) -> Result<JsonRpcResponse<U256>, Box<dyn std::error::Error>> {
    // JSON-RPC request payload
    let request_body = json!({
        "jsonrpc":"2.0",
        "method":"eth_getTransactionCount",
        "params":[address, "latest"],
        "id":1
    });
    // Create HTTP client
    let client = Client::new();

    // Send request
    let res = client
        .post(rpc_url) // replace with your RPC endpoint
        .json(&request_body)
        .send()
        .await?;

    // Parse response
    let response: JsonRpcResponse<U256> = res.json().await?;
    Ok(response)
}

async fn trace_sepolia() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // RPC endpoint
    let rpc_url_string = env::var("SEPOLIA_RPC_URL")
            .expect("SEPOLIA_RPC_URL must be set.");
    let rpc_url = rpc_url_string.as_str();

    // Transaction details
    let chain_id = 11155111;
    let from = "0xA5EaeE3738acA39334650f553Aa5BD551f0bB8cc";
    let to = "0x0000000071727De22E5E9d8BAf0edAc6f37da032";
    let data = "0x765e827f0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000a5eaee3738aca39334650f553aa5bd551f0bb8cc00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000faddcfd59924f559ac24350c4b9da44b57e6285700000000000000000000000000000000000000000000000000000000000001e0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000001400000000000000000000000000001a0ac00000000000000000000000000017219000000000000000000000000000000000000000000000000000000000000b9b400000000000000000000000000124f8000000000000000000000000000124f9b000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000204541d63c800000000000000000000000038869bf66a61cf6bdb996a6ae40d5853fd43b52600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001448d80ff0a000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000f2009a7af758ae5d7b6aae84fe4c5ba67c041dfe5336000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000246a627842000000000000000000000000faddcfd59924f559ac24350c4b9da44b57e62857009a7af758ae5d7b6aae84fe4c5ba67c041dfe5336000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000246a627842000000000000000000000000faddcfd59924f559ac24350c4b9da44b57e62857000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004d000000000000000000000000b83d1ecdecaf003c5f7666cbaab2dadf705d34afeaff0d495074c8be70af2b26547345d0721b92a913d5a1170a4c4e604221db3dd41184f94970ff2fe60299eb1c00000000000000000000000000000000000000";
    let gas_limit = 0x1f668cc;
    let gas_price = 0x10c8ea;
    let gas_priority_fee = 0x10c8e0;

    // Get current nonce from the network
    let from_nonce:u64 = match get_nonce(rpc_url, from).await? {
        JsonRpcResponse::Result(result) => {
            u64::from_str(result.result.to_string().as_str())?
        }
        JsonRpcResponse::Error(error) =>{
            return Err(error.into())
        }
    };

    let latest_block:BlockEnv;
    match get_block_by_number(rpc_url).await? {
        GetBlockByNumberResponse::Result(result) => {
            latest_block = create_block_env_from_block_details(result.result)?;
        }
        GetBlockByNumberResponse::Error(error) => {
            return Err(error.into())
        }
    };

    let prestate_tracer_response: PrestateTracerResponse = get_prestate_trace(
        rpc_url, from, to, data, format!("0x{:x}", latest_block.number)
    ).await?;
    let prestate_tracer_result = match prestate_tracer_response {
        PrestateTracerResponse::Result(result) => {
            result.result
        }
        PrestateTracerResponse::Error(error) => {
            return Err(error.into())
        }
    };

    let result = trace_transaction(
        chain_id,
        from.parse()?,
        from_nonce,
        to.parse()?,
        Bytes::from_str(data)?,
        gas_limit,
        gas_price,
        gas_priority_fee,
        latest_block,
        prestate_tracer_result
    )?;
    println!("Execution result: {:?}", result.0);
    println!("State Diff: {:?}", result.1);
    println!("Trace result: {:#}", result.2);

    Ok(())
}

async fn trace_op_sepolia() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // RPC endpoint
    let rpc_url_string = env::var("OP_SEPOLIA_RPC_URL")
            .expect("OP_SEPOLIA_RPC_URL must be set.");
    let rpc_url = rpc_url_string.as_str();

    // Transaction details
    let chain_id = 11155420; //optimism sepolia
    let from = "0xA5EaeE3738acA39334650f553Aa5BD551f0bB8cc";
    let to = "0x0000000071727De22E5E9d8BAf0edAc6f37da032";
    let data = "0x765e827f0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000a5eaee3738aca39334650f553aa5bd551f0bb8cc000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000008b88baac99f33cd29737e7771abb3c067609aaf60000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000005f0d40000000000000000000000000001ad12000000000000000000000000000000000000000000000000000000000000cb4900000000000000000000000000124f80000000000000000000000000001251d80000000000000000000000000000000000000000000000000000000000000620000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000002984e1dcf7ad4e460cfd30791ccc4f9c8a4f820ec671688f0b900000000000000000000000029fcb43b46531bca003ddc8fcb67ffe91900c7620000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000034933e00000000000000000000000000000000000000000000000000000000000001e4b63e800d000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000010000000000000000000000002dd68b007b46fbe91b9a7c3eda5a7a1063cb5b47000000000000000000000000000000000000000000000000000000000000014000000000000000000000000075cf11467937ce3f2f357ce24ffc3dbf8fd5c2260000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000084178a5fd956e624fcb61c3c2209e3dcf42c8e800000000000000000000000000000000000000000000000000000000000000648d0dc49f0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000075cf11467937ce3f2f357ce24ffc3dbf8fd5c226000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000204541d63c800000000000000000000000038869bf66a61cf6bdb996a6ae40d5853fd43b52600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001448d80ff0a000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000f2009a7af758ae5d7b6aae84fe4c5ba67c041dfe5336000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000246a6278420000000000000000000000008b88baac99f33cd29737e7771abb3c067609aaf6009a7af758ae5d7b6aae84fe4c5ba67c041dfe5336000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000246a6278420000000000000000000000008b88baac99f33cd29737e7771abb3c067609aaf6000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004d0000000000000000000000001c829b4368dda7a32ac410fd2d89c279ceff8b08ee337960610784ea8f14fe4425ad06ffa547c939aa97f014582c8d300121e1498b9f6d9a4712254a236713431b00000000000000000000000000000000000000";
    let gas_limit = 0x1f668cc;
    let gas_price = 0x10c8ea;
    let gas_priority_fee = 0x10c8e0;

    // Get current nonce from the network
    let from_nonce:u64 = match get_nonce(rpc_url, from).await? {
        JsonRpcResponse::Result(result) => {
            u64::from_str(result.result.to_string().as_str())?
        }
        JsonRpcResponse::Error(error) =>{
            return Err(error.into())
        }
    };

    let latest_block:BlockEnv;
    match get_block_by_number(rpc_url).await? {
        GetBlockByNumberResponse::Result(result) => {
            latest_block = create_block_env_from_block_details(result.result)?;
        }
        GetBlockByNumberResponse::Error(error) => {
            return Err(error.into())
        }
    };

    let prestate_tracer_response: PrestateTracerResponse = get_prestate_trace(
        rpc_url, from, to, data, format!("0x{:x}", latest_block.number)
    ).await?;
    let prestate_tracer_result = match prestate_tracer_response {
        PrestateTracerResponse::Result(result) => {
            result.result
        }
        PrestateTracerResponse::Error(error) => {
            return Err(error.into())
        }
    };

    let result = op_trace_transaction(
        chain_id,
        from.parse()?,
        from_nonce,
        to.parse()?,
        Bytes::from_str(data)?,
        gas_limit,
        gas_price,
        gas_priority_fee,
        latest_block,
        prestate_tracer_result
    )?;
    println!("Execution result: {:?}", result.0);
    println!("State Diff: {:?}", result.1);
    println!("Trace result: {:#}", result.2);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    trace_sepolia().await?;
    trace_op_sepolia().await?;
    Ok(())
}
