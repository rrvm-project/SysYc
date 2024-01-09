use std::{collections::HashMap, vec};

use llvm::{LlvmInstrTrait, CloneLlvmInstr, ArithInstr, Value, Temp, CompInstr};
use rrvm::program::{*, self};
use utils::{UseTemp, Label, label};

#[derive(Debug, Clone, Copy)]
pub enum StackValue{
    Int(i32),
    Float(f32),
    IntPtr(usize),
    FloatPtr(usize)
}

impl Default for StackValue {
    fn default() -> Self {
        StackValue::Int(0)
    }
}

impl From<i32> for StackValue {
    fn from(value: i32) -> Self {
        StackValue::Int(value)
    }
}

impl From<f32> for StackValue {
    fn from(value: f32) -> Self {
        StackValue::Float(value)
    }
}

impl StackValue{
    pub fn as_i32(&self) -> i32{
        match self {
            StackValue::Int(v) => *v,
            _ => unreachable!()
        }
    }

    pub fn as_f32(&self) -> f32{
        match self {
            StackValue::Float(v) => *v,
            _ => unreachable!()
        }
    }
}

struct FuncStackFrame{
    pub ra: Option<usize>,
    pub name: String,
    pub fp: usize,
    pub last_label: String,
    pub current_label: String,
    pub temp : HashMap<String, StackValue>,
    pub return_to: String
}

struct GetReturnValue{
    target: llvm::Temp
}


pub struct MiddleSimulator{
    pub input: String,
    pub input_position: usize,
    pub output: Vec<String>,
    pub step_count: usize,
    pub instr_list: Vec<Box<dyn LlvmInstrTrait>>,
    pub label_map: HashMap<String, HashMap<String, usize>>,
    pub return_scratch: Option<StackValue>,
    pub pc: usize,
    pub memory_stack: Vec<StackValue>,
    pub calling_stack: Vec<FuncStackFrame>,
    pub calling_params: HashMap<String, Vec<llvm::Value>>
}

impl MiddleSimulator {
    pub fn new(input : String) -> Self {
        MiddleSimulator{
            input,
            input_position : 0,
            output: vec![],
            step_count: 0,
            instr_list: vec![],
            label_map: HashMap::new(),
            return_scratch: None,
            pc: 0,
            memory_stack: vec![],
            calling_stack: vec![],
            calling_params: HashMap::new()
        }
    }

    fn init(&mut self, program: &LlvmProgram){
        self.instr_list = vec![];
        for item in &program.funcs{
            self.calling_params.insert(item.name.clone(), item.params.clone());
            let mut labels = HashMap::new();
            for block in &item.cfg.blocks{
                let label = block.borrow().label();
                let current_addr = self.instr_list.len();
                labels.insert(label.name.clone(), current_addr);

                for item in &block.borrow().phi_instrs{
                    self.instr_list.push(item.clone_box());
                }
                for item in &block.borrow().instrs{
                    self.instr_list.push(item.clone_box());
                }
                for item in &block.borrow().jump_instr{
                    self.instr_list.push(item.clone_box());
                }
            }

            self.label_map.insert(item.name.clone(), labels);
        }
        
        println!("{:?}", &self.label_map);

    }
    pub fn run_program(&mut self,  program : &LlvmProgram) {
        self.init(program);

        self.pc = *self.label_map.get("main").unwrap().get("entry").to_owned().unwrap();
        dbg!(self.pc);

        self.calling_stack.push(FuncStackFrame{
            ra: None,
            name: "main".into(),
            fp: self.memory_stack.len(),
            current_label: "entry".into(),
            last_label: "".into(),
            temp: HashMap::new(),
            return_to: "".to_string()
        });

        loop {
            let instr = self.instr_list.get(self.pc).unwrap();
            println!("{:?} {:#}",self.pc, &instr);
            
            let mut next = self.pc + 1;
            let frame = self.calling_stack.last_mut().unwrap();
            println!("last {:?} now {:?}", frame.last_label, frame.current_label);
            println!("{:?}", frame.temp);
            let labels = self.label_map.get(&frame.name).unwrap();
            let mut return_info: Option<(String, StackValue)> = None;

            match instr.get_variant() {
                llvm::LlvmInstrVariant::ArithInstr(instr) => do_arith_instr(instr, frame),
                llvm::LlvmInstrVariant::CompInstr(instr) => do_comp_instr(instr, frame),
                llvm::LlvmInstrVariant::ConvertInstr(_) => todo!(),
                llvm::LlvmInstrVariant::JumpInstr(instr) => {
                    frame.last_label = frame.current_label.clone();
                    frame.current_label = instr.target.name.clone();
                    
                    next = *labels.get(&instr.target.name).unwrap();
                },
                llvm::LlvmInstrVariant::JumpCondInstr(instr) => {
                    let value = get_stack(&instr.cond, frame);

                    let jump = match instr.var_type {
                        llvm::VarType::I32 => value.as_i32() != 0,
                        llvm::VarType::F32 => value.as_f32() != 0f32,
                        _ => todo!()
                    };

                    if jump {
                        frame.last_label = frame.current_label.clone();
                        frame.current_label = instr.target_true.name.clone();
                        next = *labels.get(&instr.target_true.name).unwrap();
                    } else {
                        frame.last_label = frame.current_label.clone();
                        frame.current_label = instr.target_false.name.clone();
                        next = *labels.get(&instr.target_false.name).unwrap();
                    }

                },
                llvm::LlvmInstrVariant::PhiInstr(instr) => {
                    for (value, label) in &instr.source {
                        if label.name == frame.last_label{
                            frame.temp.insert(instr.target.name.clone(), get_stack(value, frame));
                            break;
                        }
                    }
                },
                llvm::LlvmInstrVariant::RetInstr(instr) => {
                    if let Some(value) = &instr.value {
                        self.return_scratch = Some(get_stack(value, frame));
                    } else {
                        self.return_scratch = None;
                    }
                    
                    self.memory_stack.resize(frame.fp, StackValue::Int(0));
                    

                    if let Some(ra) = &frame.ra {
                        next = *ra;
                    } else {
                        return;
                    }

                    // self.calling_stack.get(self.calling_stack.len() -2 ).unwrap().temp.insert(frame.return_to.clone(), self.return_scratch.unwrap_or_default());
                    return_info = Some((frame.return_to.clone(), self.return_scratch.unwrap_or_default()));

                },
                llvm::LlvmInstrVariant::AllocInstr(_) => todo!(),
                llvm::LlvmInstrVariant::StoreInstr(_) => todo!(),
                llvm::LlvmInstrVariant::LoadInstr(_) => todo!(),
                llvm::LlvmInstrVariant::GEPInstr(_) => todo!(),
                llvm::LlvmInstrVariant::CallInstr(instr) => {
                    
                    let mut value_list = vec![];
                    let mut argument_name_list = vec![];
                    for (_type, value) in &instr.params{
                        value_list.push(get_stack(value, frame));
                    }
                    for item in self.calling_params.get(&instr.func.name).unwrap(){
                        match item{
                            Value::Temp(t) => argument_name_list.push(t.name.clone()),
                            _ => unreachable!()
                        }
                    }

                    assert_eq!(argument_name_list.len(), value_list.len());

                    let arguments: HashMap<String, StackValue> = argument_name_list.into_iter().zip(value_list.into_iter()).collect();

                    dbg!(&arguments);

                    self.calling_stack.push(FuncStackFrame{
                        ra: Some(self.pc + 1),
                        name: instr.func.name.clone(),
                        fp: self.memory_stack.len(),
                        current_label: "entry".into(),
                        last_label: "".into(),
                        temp: arguments,
                        return_to: instr.target.name.clone()
                    });

                    next = *self.label_map.get(&instr.func.name).unwrap().get("entry".into()).unwrap();
                    
                },
            }
            self.pc = next;

            if let Some((name, value)) = return_info {
                self.calling_stack.pop();
                self.calling_stack.last_mut().unwrap().temp.insert(name, value);
            }
        }
    }
}

fn get_stack(value: &llvm::Value, frame: &FuncStackFrame) -> StackValue{
    match value {
        llvm::Value::Int(v) => StackValue::Int(*v),
        llvm::Value::Float(v) => StackValue::Float(*v),
        llvm::Value::Temp(t) => {
            if let Some(v) = frame.temp.get(&t.name){
                v.clone()
            } else {
                unreachable!();
            }
        },
    }
}

fn do_arith_instr(instr: &ArithInstr, frame: &mut FuncStackFrame){

    let lhs = get_stack(&instr.lhs, frame);
    let rhs = get_stack(&instr.rhs, frame);
    
    let value: StackValue = match instr.op{
        llvm::ArithOp::Add => lhs.as_i32().wrapping_add(rhs.as_i32()).into(),
        llvm::ArithOp::Sub => lhs.as_i32().wrapping_sub(rhs.as_i32()).into(),
        llvm::ArithOp::Div => lhs.as_i32().wrapping_div(rhs.as_i32()).into(),
        llvm::ArithOp::Mul => lhs.as_i32().wrapping_mul(rhs.as_i32()).into(),
        llvm::ArithOp::Rem => lhs.as_i32().wrapping_rem(rhs.as_i32()).into(),
        llvm::ArithOp::Fadd => (lhs.as_f32() + rhs.as_f32()).into(),
        llvm::ArithOp::Fsub => (lhs.as_f32() - rhs.as_f32()).into(),
        llvm::ArithOp::Fdiv => (lhs.as_f32() / rhs.as_f32()).into(),
        llvm::ArithOp::Fmul => (lhs.as_f32() * rhs.as_f32()).into(),
        llvm::ArithOp::Shl => todo!(),
        llvm::ArithOp::Lshr => todo!(),
        llvm::ArithOp::Ashr => todo!(),
        llvm::ArithOp::And => todo!(),
        llvm::ArithOp::Or => todo!(),
        llvm::ArithOp::Xor => todo!(),
        llvm::ArithOp::AddD => todo!(),
    };

    let target = &instr.target;
    frame.temp.insert(target.name.clone(), value);

}

fn do_comp_instr(instr: &CompInstr, frame: &mut FuncStackFrame){

    let lhs = get_stack(&instr.lhs, frame);
    let rhs = get_stack(&instr.rhs, frame);

    let value: bool = match instr.var_type {
        llvm::VarType::I32 => {
            match instr.op{
                llvm::CompOp::EQ => lhs.as_i32() == rhs.as_i32(),
                llvm::CompOp::NE => lhs.as_i32() != rhs.as_i32(),
                llvm::CompOp::SGT => lhs.as_i32() > rhs.as_i32(),
                llvm::CompOp::SGE => lhs.as_i32() >= rhs.as_i32(),
                llvm::CompOp::SLT => lhs.as_i32() < rhs.as_i32(),
                llvm::CompOp::SLE => lhs.as_i32() <= rhs.as_i32(),
                llvm::CompOp::OEQ => todo!(),
                llvm::CompOp::ONE => todo!(),
                llvm::CompOp::OGT => todo!(),
                llvm::CompOp::OGE => todo!(),
                llvm::CompOp::OLT => todo!(),
                llvm::CompOp::OLE => todo!(),
            }
        },
        llvm::VarType::F32 => {
            match instr.op{
                llvm::CompOp::EQ => lhs.as_f32() == rhs.as_f32(),
                llvm::CompOp::NE => lhs.as_f32() != rhs.as_f32(),
                llvm::CompOp::SGT => lhs.as_f32() > rhs.as_f32(),
                llvm::CompOp::SGE => lhs.as_f32() >= rhs.as_f32(),
                llvm::CompOp::SLT => lhs.as_f32() < rhs.as_f32(),
                llvm::CompOp::SLE => lhs.as_f32() <= rhs.as_f32(),
                llvm::CompOp::OEQ => todo!(),
                llvm::CompOp::ONE => todo!(),
                llvm::CompOp::OGT => todo!(),
                llvm::CompOp::OGE => todo!(),
                llvm::CompOp::OLT => todo!(),
                llvm::CompOp::OLE => todo!(),
            }
        },
        _ => unreachable!()
    };
    
    

    let target = &instr.target;
    frame.temp.insert(target.name.clone(), (value as i32).into());

}