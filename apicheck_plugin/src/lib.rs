#![feature(plugin_registrar)]
#![feature(box_syntax, rustc_private)]

extern crate libapicheck;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;
extern crate syntax_pos;

use rustc::hir;
use rustc::lint::{EarlyContext, EarlyLintPassObject, LateContext, LateLintPassObject, LintArray, LintContext, LintPass};
use rustc_plugin::Registry;
use syntax::ast;

use libapicheck::config::Config;

lazy_static! {
    static ref CONFIG: Config = {
        Config::default()
    };
}

declare_lint!(TEST_LINT, Warn, "Warn about items named 'lintme'");

struct EarlyPass;
struct LatePass;

impl LintPass for LatePass {
   fn name(&self) -> &'static str { "apicheck" }

   fn get_lints(&self) -> LintArray {
       lint_array!() // We'll get to this later, kind of...
   }
}

impl LintPass for EarlyPass {
   fn name(&self) -> &'static str { "apicheck" }

   fn get_lints(&self) -> LintArray {
       lint_array!(TEST_LINT)
   }
}

#[plugin_registrar]
pub fn register_plugins(reg: &mut Registry) {
    println!("registry session");
    println!("    working_dir: {:?}", reg.sess.working_dir);
    println!("    local_crate_source_file: {:?}", reg.sess.local_crate_source_file);
    println!("    option prints: {:?}", reg.sess.opts.prints);
    reg.register_early_lint_pass(box EarlyPass as EarlyLintPassObject);
    reg.register_late_lint_pass(box LatePass as LateLintPassObject);
}

impl rustc::lint::EarlyLintPass for EarlyPass {
    fn check_item(&mut self, cx: &EarlyContext, it: &ast::Item) {
        println!("Early pass, item {:#?}", it);
        if it.ident.name.as_str() == "lintme" {
            cx.span_lint(TEST_LINT, it.span, "item is named 'lintme'");
        }
        if let Some(js) = libapicheck::check_item(&it, &CONFIG) {
            println!("json: {}", js.pretty(2));
        }
    }
}

impl<'a, 'tcx> rustc::lint::LateLintPass<'a, 'tcx> for LatePass {
    fn check_expr(&mut self, _cx: &LateContext<'a, 'tcx>, expr: &'tcx hir::Expr) {
        println!("Late pass, expression: {:?}", expr);
    }
}
