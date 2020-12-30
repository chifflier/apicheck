use rustc_ast::ast;
use rustc_span::Symbol;

pub(crate) fn contains_name(attrs: &[ast::Attribute], name: Symbol) -> bool {
    attrs.iter().any(|attr| attr.has_name(name))
}

pub(crate) fn first_attr_value_str_by_name(
    attrs: &[ast::Attribute],
    name: Symbol,
) -> Option<Symbol> {
    attrs
        .iter()
        .find(|attr| attr.has_name(name))
        .and_then(|attr| attr.value_str())
}

pub(crate) trait MetaVisitor<'ast> {
    fn visit_meta_item(&mut self, meta_item: &'ast ast::MetaItem) {
        match meta_item.kind {
            ast::MetaItemKind::Word => self.visit_meta_word(meta_item),
            ast::MetaItemKind::List(ref list) => self.visit_meta_list(meta_item, list),
            ast::MetaItemKind::NameValue(ref lit) => self.visit_meta_name_value(meta_item, lit),
        }
    }

    fn visit_meta_list(
        &mut self,
        _meta_item: &'ast ast::MetaItem,
        list: &'ast [ast::NestedMetaItem],
    ) {
        for nm in list {
            self.visit_nested_meta_item(nm);
        }
    }

    fn visit_meta_word(&mut self, _meta_item: &'ast ast::MetaItem) {}

    fn visit_meta_name_value(&mut self, _meta_item: &'ast ast::MetaItem, _lit: &'ast ast::Lit) {}

    fn visit_nested_meta_item(&mut self, nm: &'ast ast::NestedMetaItem) {
        match nm {
            ast::NestedMetaItem::MetaItem(ref meta_item) => self.visit_meta_item(meta_item),
            ast::NestedMetaItem::Literal(ref lit) => self.visit_literal(lit),
        }
    }

    fn visit_literal(&mut self, _lit: &'ast ast::Lit) {}
}
