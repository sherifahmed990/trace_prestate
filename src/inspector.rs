use revm::{interpreter::{CallInputs, CallOutcome, CreateInputs, CreateOutcome, Interpreter, InterpreterTypes}, primitives::{HashMap, Log}, Inspector};
use serde_json::{json, Value};

pub struct MyInspector<'a> {
    pub gas_used: u64,
    pub call_count: usize,
    pub trace_stack: &'a mut Vec<(u64, Value)>,
    logs_stack: Vec<Log>,
    current_depth: u64,
}

impl<'a> MyInspector<'a> {
    // Constructor function
    pub fn new(trace_stack: &'a mut Vec<(u64, Value)>) -> Self {
       Self{
           gas_used: 0,
           call_count: 0,
           trace_stack,
           logs_stack: Vec::<Log>::new(),
           current_depth: 0
       }
    }
}
impl<'a, CTX, INTR: InterpreterTypes> Inspector<CTX, INTR> for MyInspector<'a> {
    fn step(&mut self, interp: &mut Interpreter<INTR>, _context: &mut CTX) {
        self.gas_used += interp.gas.spent();
    }
    fn create(&mut self, context: &mut CTX, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        self.call_count += 1;
        self.current_depth += 1;
        None // Don't override the call
    }
    fn create_end(&mut self, context: &mut CTX, inputs: &CreateInputs, outcome: &mut CreateOutcome) {
        let calls = &mut Vec::new();

        while let Some(stack_top) = self.trace_stack.pop(){
            if stack_top.0 == self.current_depth {
                self.trace_stack.push(stack_top);
                break;
            }
            else if stack_top.0 == self.current_depth + 1 {
                calls.push(stack_top.1);
            }
            else{
                assert!(
                    !(stack_top.0 > self.current_depth),
                    "Invalid trace stack. stack top depth: {}, current depth: {}",
                    stack_top.0, self.current_depth
                );
            }
        }

        let call_info = json!({
            "isCreate": "true",
            "inputs": inputs,
            "outcome": outcome,
            "calls": calls,
            "logs": self.logs_stack
        });

        self.logs_stack.clear();
        self.trace_stack.push((self.current_depth, call_info));
        self.current_depth -= 1;
     }
   
    fn call(&mut self, _context: &mut CTX, _inputs: &mut CallInputs) -> Option<CallOutcome> {
        self.call_count += 1;
        self.current_depth += 1;
        None // Don't override the call
    }
     fn call_end(&mut self, context: &mut CTX, inputs: &CallInputs, outcome: &mut CallOutcome) {
        let calls = &mut Vec::new();

        while let Some(stack_top) = self.trace_stack.pop(){
            if stack_top.0 == self.current_depth {
                self.trace_stack.push(stack_top);
                break;
            }
            else if stack_top.0 == self.current_depth + 1 {
                calls.push(stack_top.1);
            }
            else{
                assert!(
                    !(stack_top.0 > self.current_depth),
                    "Invalid trace stack. stack top depth: {}, current depth: {}",
                    stack_top.0, self.current_depth
                );
            }
        }

        let call_info = json!({
            "inputs": inputs,
            "outcome": outcome,
            "calls": calls,
            "logs": self.logs_stack
        });

        self.logs_stack.clear();
        self.trace_stack.push((self.current_depth, call_info));
        self.current_depth -= 1;
     }
    fn log(&mut self, _interp: &mut Interpreter<INTR>, _ctx: &mut CTX, log: Log) {
        self.logs_stack.push(log);
    }
}
