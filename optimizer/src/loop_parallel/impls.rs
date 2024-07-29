
use std::{cell::RefCell, collections::{HashMap, HashSet, VecDeque}, rc::Rc};

use super::LoopParallel;
use crate::RrvmOptimizer;
use llvm::{ArithInstr, ArithOp, CallInstr, CompOp, JumpInstr, LlvmInstr, LlvmInstrTrait, LlvmInstrVariant, LlvmTemp, LlvmTempManager, Value, VarType};
use rrvm::{
	func::Entrance, prelude::LlvmBasicBlock, program::{LlvmFunc, LlvmProgram}, rrvm_loop::loop_analysis, LlvmNode
};
use utils::{errors::Result, math::increment, Label};

struct PointerTracer{
    ptr_set: HashMap<LlvmTemp,u32>,
    named : HashMap<String, u32>,
    read: HashSet<u32>,
    write: HashSet<u32>,
    last: u32
}

impl PointerTracer{
    pub fn new()->Self{
        Self{ptr_set:HashMap::new(), last:0, read: HashSet::new(), write: HashSet::new(),named: HashMap::new()}
    }
    pub fn get(&mut self, ptr :&LlvmTemp) -> u32{

        *self.ptr_set.entry(ptr.clone()).or_insert_with(||{0})

    }

    pub fn create(&mut self, ptr :&LlvmTemp) -> u32{
        *self.ptr_set.entry(ptr.clone()).or_insert_with(||{self.last+=1;self.last})
    }

    pub fn name(&mut self, ptr :&LlvmTemp, ident: &String) -> u32{
        // *self.named.entry(ident.clone()).or_insert_with(||self.create(ptr))
        if let Some(id) = self.named.get(ident){
            *id
        } else {
            let id = self.create(ptr);
            self.named.insert(ident.clone(), id);
            id
        }
        
    }

    pub fn link(&mut self, src : &LlvmTemp, dst:&LlvmTemp) ->u32{
        let c = self.get(dst);
        self.ptr_set.insert(src.clone(), c);
        c
    }

    pub fn clear(&mut self){
        self.read.clear();
        self.write.clear();
    }

    pub fn read(&mut self, a : &LlvmTemp) -> bool{
        let c = self.get(a);
        self.read.insert(c);
        self.write.contains(&c) || c == 0
    }

    pub fn write(&mut self, a : &LlvmTemp) -> bool{
        let c = self.get(a);
        self.write.insert(c);
        self.read.contains(&c) || c == 0
    }
}
fn process_func(func: &mut LlvmFunc, mgr: &mut LlvmTempManager) -> bool {

    //传入参数的数组默认可能来自同一个数组。mm测例中经过内联能判断出属于不同数组。

    //ptr set:
    let mut ptr_set  = PointerTracer::new();
    
    for block in &func.cfg.blocks{
        for instr in &block.borrow().instrs{
            match instr.get_variant () {
                llvm::LlvmInstrVariant::AllocInstr(i) => {
                    ptr_set.create(&i.target);
                },
                llvm::LlvmInstrVariant::StoreInstr(i) => {
                    
                    if let Some(t) = i.addr.get_temp_ref(){
                        ptr_set.get(t);
                    }
                },
                llvm::LlvmInstrVariant::LoadInstr(i) => {
                    
                    if let Some(t) = i.addr.get_temp_ref(){
                        if t.is_global{
                            ptr_set.name(t, &t.name);
                        }
                        if i.target.var_type.is_ptr(){
                            ptr_set.link(&i.target, t);
                        }
                    }
                },
                llvm::LlvmInstrVariant::GEPInstr(i) => {
                    if let Some(t) = i.addr.get_temp_ref(){
                        ptr_set.link(&i.target, t);
                    }
                },
                llvm::LlvmInstrVariant::CallInstr(i) => {
                    if i.target.var_type.is_ptr(){
                        //since function that returns ptr can only be our function to fill zeros, which returns the ptr in same array
                        for (t, value) in i.params.iter(){
                            if t.is_ptr(){
                                ptr_set.link(&i.target, value.get_temp_ref().unwrap());
                            }
                        }
                    }
                },
                _ => {}

            }
        }
    }


	let mut loop_map = HashMap::new();
	for top_loop in &func.cfg.loop_analysis(&mut loop_map).borrow().subloops {
		let head = &top_loop.borrow().header;

		if head.borrow().phi_instrs.len() != 1 {
			continue;
		}
		let (loop_index, v0, prev0, v1, prev1) = {
			let phi = &head.borrow().phi_instrs[0];
			if phi.source.len() == 2 && head.borrow().prev.len() == 2 {
				let ((v0, l0), (v1, l1)) =
					(phi.source.get(0).unwrap(), phi.source.get(1).unwrap());
				if *l0 == head.borrow().prev[0].borrow().label()
					&& *l1 == head.borrow().prev[1].borrow().label()
				{
					(
						phi.target.clone(),
						v0.clone(),
						head.borrow().prev[0].clone(),
						v1.clone(),
						head.borrow().prev[1].clone(),
					)
				} else if *l1 == head.borrow().prev[0].borrow().label()
					&& *l0 == head.borrow().prev[1].borrow().label()
				{
					(
						phi.target.clone(),
						v0.clone(),
						head.borrow().prev[1].clone(),
						v1.clone(),
						head.borrow().prev[0].clone(),
					)
				} else {
					continue;
				}
			} else {
				continue;
			}
		};

		let loop0 = if let Some(loop0) = loop_map.get(&prev0.borrow().id) {
			loop0
		} else {
			continue;
		};
		let loop1 = if let Some(loop1) = loop_map.get(&prev1.borrow().id) {
			loop1
		} else {
			continue;
		};

		let mut v_start = None;
		let mut block_prev = None;
		let mut v_update = None;

		let mut assign = |vi, loopi, prev| {
			if top_loop.borrow().is_super_loop_of(loopi) {
				v_update = Some(vi);
			} else if !top_loop.borrow().is_super_loop_of(loopi) {
				v_start = Some(vi);
				block_prev = Some(prev);
			}
		};

		assign(v0, loop0, prev0);
		assign(v1, loop1, prev1);

		let (v_start, block_prev, v_update) =
			if (v_start.is_some() && block_prev.is_some() && v_update.is_some()) {
				(v_start.unwrap(), block_prev.unwrap(), v_update.unwrap())
			} else {
				continue;
			};

		


        let v_update = match v_update {
            Value::Temp(t) => t,
            _ => {continue;}
        };

        
        let mut exit_op : Option<CompOp> = None;
        let mut exit_value : Option<Value> = None;
        let mut exit: Option<LlvmNode> = None;
        let mut stack_move = false;
        let mut array_dependance = false;
        let mut invalid_exit = false;
        let mut invalid_index = false;
        let head_id = head.borrow().id;

        let mut update_value: Option<i32> = None;  
        let mut loop_when_op_hold : Option<bool> = None;
        
        ptr_set.clear();
		'block : for block in &top_loop.borrow().blocks(&func.cfg, &loop_map) {
			//检查是否单出口， 注意这里的block包括head
            if block.borrow().id == head_id{

                let (comp_target, jumptrue, jumpfalse) = match block.borrow().jump_instr.as_ref().map(|i|i.get_variant()){
                    Some(LlvmInstrVariant::JumpCondInstr(j)) =>{
                        (j.cond.clone().get_temp(), j.target_true.clone(), j.target_false.clone())
                    },
                    _ =>{
                        invalid_exit = true;
                        break 'block;
                    }
                };

                if comp_target.is_none(){
                    invalid_exit = true;
                    break 'block; 
                }

                let comp_target = comp_target.unwrap();

                
                for instr in block.borrow().instrs.iter(){
                    match instr.get_variant(){
                        llvm::LlvmInstrVariant::CompInstr(comp) if comp.target == comp_target => {
                            match (&comp.lhs, &comp.rhs) {
                                (Value::Temp(t) , rhs) if *t == loop_index =>{
                                    match rhs{
                                        Value::Temp(r_tmp) if !block.borrow().live_in.contains(r_tmp) => {
                                            invalid_index = true;
                                            break 'block;
                                        }  
                                        _ => {
                                            exit_value = Some(rhs.clone());
                                            exit_op = Some(comp.op)
                                        }
                                    }
                                },
                                (lhs, Value::Temp(t)) if *t == loop_index => {
                                    match lhs{
                                        Value::Temp(l_tmp) if !block.borrow().live_in.contains(l_tmp) => {
                                            invalid_index = true;
                                            break 'block;
                                        }  
                                        _ => {
                                            exit_value = Some(lhs.clone());
                                            exit_op = Some(comp.op.reverse_lhs_rhs())
                                        }
                                    }
                                }
                                _ => {
                                    invalid_index = true;
                                    break 'block;
                                }
                            }
                        },
                        _ => {}
                    }
                }

                for succ in block.borrow().succ.iter(){
                    let id = succ.borrow().id;
                    let exit_loop = loop_map.get(&id);
                    if  exit_loop.is_some_and(|exit|{
                        ! top_loop.borrow().is_super_loop_of(exit)
                    }){
                        if exit.is_some(){
                            invalid_exit = true;
                            break 'block;
                        }

                        if succ.borrow().label() == jumptrue{
                            loop_when_op_hold = Some(false);
                        } else if succ.borrow().label() == jumpfalse {
                            loop_when_op_hold = Some(true);
                        }

                        exit = Some(succ.clone())
                        
                    }  
                }
            } else {
                for succ in block.borrow().succ.iter(){
                    let id = succ.borrow().id;
                    let exit_loop = loop_map.get(&id);
                    if  exit_loop.is_some_and(|exit|{
                        top_loop.borrow().is_super_loop_of(exit)
                    }){
                        continue;
                    }  
                    invalid_exit = true;
                    break 'block;
                }
            }
			//检查读写正交性
			//检查是否有函数调用（数组初始化也算，由于后端指令在caller save时可能动栈）
            for instr in block.borrow().instrs.iter(){
                match instr.get_variant() {
                    llvm::LlvmInstrVariant::ArithInstr(i) if i.target == v_update => {
                        if i.op == ArithOp::Add{
                            match (&i.lhs, &i.rhs) {
                                (Value::Temp(t), Value::Int(i)) if *t == loop_index => {
                                    update_value = Some(*i);
                                    
                                },
                                (Value::Int(i), Value::Temp(t)) if *t == loop_index => {
                                    update_value = Some(*i);
                                },
                                _ => {
                                    invalid_index = true;
                                    break 'block;
                                }
                            }
                        } else if i.op == ArithOp::Sub {
                            match (&i.lhs, &i.rhs) {
                                (Value::Temp(t), Value::Int(i)) if *t == loop_index => {
                                    update_value = Some(- *i);
                                },
                                _ => {
                                    invalid_index = true;
                                    break 'block;
                                }
                            }
                        }
                        
                    }

                    llvm::LlvmInstrVariant::AllocInstr(_) | llvm::LlvmInstrVariant::CallInstr(_) => {
                        stack_move = true;
                        break 'block;
                    },

                    llvm::LlvmInstrVariant::StoreInstr(i) => {
                        if ptr_set.write(i.addr.get_temp_ref().unwrap()){
                            array_dependance = true;
                            break 'block;
                        }
                    },
                    llvm::LlvmInstrVariant::LoadInstr(i) => {
                        if i.addr.get_temp_ref().unwrap().is_global{continue;}
                        if ptr_set.read(i.addr.get_temp_ref().unwrap()){
                            array_dependance = true;
                            break 'block;
                        }
                    },
                    _ => {}
                }
            }
			//by the way看看现在是否把调用了数组初始化的函数当作纯函数
		}


        if array_dependance || stack_move || invalid_exit || invalid_index{
            continue;
        }

        if loop_index.var_type != VarType::I32 {
            continue;
        }
        
        
     
       
        if let (Some(exit), Some(exit_op), Some(exit_value), Some(update_value), Some(loop_when_op_hold)) = (exit, exit_op, exit_value, update_value, loop_when_op_hold) {
            // println!("======================================");
            // dbg!(head.borrow().id);
            // dbg!(&loop_index, &v_start, block_prev.borrow().id);
            // dbg!(exit_op, exit_value, update_value, loop_when_op_hold);
            // println!("======================================");

            if match exit_op {
                CompOp::SGT | CompOp::SGE | CompOp::SLT | CompOp::SLE => false,
                _ => true
            } {
                continue;
            }

            let max_step = 5_3687_0911;

            if update_value > max_step || update_value < -max_step{
                // overflow
                continue;
            }


            let mut ok = false;

            for instr in head.borrow_mut().instrs.iter_mut(){
                if let Some(target)  = instr.get_write(){
                    if target == v_update{
                        *instr = Box::new(
                            ArithInstr{
                                target,
                                op: ArithOp::Add,
                                var_type: loop_index.var_type,
                                lhs: Value::Temp(loop_index.clone()),
                                rhs: Value::Int(4 * update_value),
                            }
                        );
                        ok = true;
                    }
                }
            }

            if !ok {
                continue;
            }

            // By here, must loop parallel!

         


            let weight = head.borrow().weight;
            		//研究怎样插入基本块
            let new_node =  LlvmBasicBlock::new(increment(&mut func.total), weight);
            let new_node = Rc::new(RefCell::new(new_node));


            let mut instrs: Vec<LlvmInstr> = vec![];

            let tid = mgr.new_temp(llvm::VarType::I32, false);

            instrs.push(Box::new(
                CallInstr{
                    target: tid.clone(),
                    var_type: llvm::VarType::I32,
                    func: Label{name: "__create_threads".to_string()},
                    params: vec![(VarType::I32, Value::Int(4))],
                }
            ));


            let offset = mgr.new_temp(VarType::I32, false);
            instrs.push(Box::new(
                ArithInstr{
                    target: offset.clone(),
                    var_type: llvm::VarType::I32,
                    op: ArithOp::Mul,
                    lhs: Value::Temp(tid.clone()),
                    rhs: Value::Int(update_value)
                }
            ));

            let new_start = mgr.new_temp(llvm::VarType::I32, false);
            instrs.push(Box::new(
                ArithInstr{
                    target: new_start.clone(),
                    var_type: llvm::VarType::I32,
                    op: ArithOp::Add,
                    lhs: v_start,
                    rhs: Value::Temp(offset)
                }
            ));

            let prev_label = block_prev.borrow().label();
            let header_label = head.borrow().label();
            let new_in_label = new_node.borrow().label();


            new_node.borrow_mut().jump_instr = Some({
                Box::new(
                    JumpInstr{
                        target: header_label.clone(),
                    }
                )
            });

            new_node.borrow_mut().instrs = instrs;

            func.cfg.blocks.push(new_node.clone());


            let mut label_map = HashMap::new();
            label_map.insert(header_label, new_in_label.clone());


            
            block_prev.borrow_mut().jump_instr.as_mut().map(
                |instr|{
                    instr.map_label(&label_map);
                }
            );

            block_prev.borrow_mut().succ.iter_mut().for_each(|block|{
                if block.borrow().id == head_id {
                    *block = new_node.clone()
                }
            });

            //前后连起来

            new_node.borrow_mut().prev = vec![block_prev.clone()];

            new_node.borrow_mut().succ = vec![head.clone()];

            head.borrow_mut().prev.iter_mut().for_each(|block|{
                if block.borrow().id == block_prev.borrow().id {
                    *block = new_node.clone()
                }
            });


            head.borrow_mut().phi_instrs.iter_mut().for_each(|phi|{
                phi.map_label(&label_map);
                if phi.target == loop_index{
                    for (value, label) in phi.source.iter_mut(){
                        if *label == prev_label{
                            *value = Value::Temp(new_start.clone());
                            *label = new_in_label.clone();
                        }
                    }
                }
            });

    
            exit.borrow_mut().instrs.insert(0, Box::new(
                CallInstr{
                    target: mgr.new_temp(VarType::Void, false),
                    var_type: VarType::Void,
                    func: Label{name: "__join_threads".to_string()},
                    params: vec![(VarType::I32, Value::Temp(tid)), (VarType::I32, Value::Int(4))],
                }
            ));
            

        }
	}

	false
}

impl RrvmOptimizer for LoopParallel {
	fn new() -> Self {
		Self {}
	}

	fn apply(self, program: &mut LlvmProgram) -> Result<bool> {
        program.analysis();
        let mgr = &mut program.temp_mgr;
		Ok(program.funcs.iter_mut().fold(false, |x, func| x | process_func(func, mgr)))
        
	}
}
