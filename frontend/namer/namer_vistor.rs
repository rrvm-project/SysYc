pub struct NamerVisitor {

}

impl Visitor for NamerVisitor {
  fn visitProgram(&self, program: &mut Program, ctx: &mut dyn Scope) {
    program.accept(self, ctx)
  }
}