//! Native rules: checks a pattern can't express, implemented in Rust and
//! registered by id (SR-ARCHITECTURE decision 15b). The rule's Markdown
//! file is still the rule — its docs, fix class, options and examples —
//! and names one of these by `native: <id>`.
//!
//! Every native rule declares the node kinds it inspects, so the engine
//! hands it only those (decision 15); it is deterministic (L5) and the
//! engine isolates it on panic (decision 25).

mod accessors;
mod classes;
mod conventions;
mod css_atrules;
mod css_declarations;
mod css_media;
mod css_misc;
mod css_selectors;
mod css_syntax;
mod css_values;
mod data_json;
mod data_md;
mod data_toml;
mod data_yaml;
mod duplicates;
mod expressions;
mod literals;
mod no_undef;
mod no_unreachable;
mod node;
mod react;
mod regexp;
mod stencil;
mod typescript;
mod unused_vars;
mod useless_assignment;
mod useless_escape;
mod util;

use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use codelaw_semantic::SemanticModel;
use std::collections::BTreeMap;
use std::sync::Arc;

/// What a native rule sees of one file.
pub struct NativeContext<'a> {
    pub source: &'a str,
    pub language: Language,
    pub semantic: Option<&'a SemanticModel>,
    /// The rule's options, resolved for this file.
    pub options: &'a BTreeMap<String, serde_json::Value>,
    /// The rule's pack's data tables (SR-ARCHITECTURE decision 15c).
    pub data: &'a BTreeMap<String, Arc<serde_json::Value>>,
}

/// One finding of a native rule. `edits` are offered when the rule file's
/// fix class is `auto` or `suggest`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeFinding {
    pub start: usize,
    pub end: usize,
    pub message: String,
    pub edits: Vec<TextEdit>,
}

pub trait NativeRule: Send + Sync {
    fn id(&self) -> &'static str;
    fn languages(&self) -> &'static [Language];
    /// The node kinds `check` is called on.
    fn kinds(&self) -> &'static [&'static str];
    fn needs_semantic(&self) -> bool {
        false
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>);
}

const JS_ONLY: &[Language] = &[Language::Javascript, Language::Jsx];
const TS_ONLY: &[Language] = &[Language::Typescript, Language::Tsx];

static JS_NO_UNUSED_VARS: unused_vars::NoUnusedVars = unused_vars::NoUnusedVars { id: "js/no-unused-vars", languages: JS_ONLY };
static TS_NO_UNUSED_VARS: unused_vars::NoUnusedVars = unused_vars::NoUnusedVars { id: "ts/no-unused-vars", languages: TS_ONLY };
static TS_NO_UNUSED_EXPRESSIONS: typescript::NoUnusedExpressions = typescript::NoUnusedExpressions { id: "ts/no-unused-expressions", languages: TS_ONLY };

/// Every native rule; ids are unique, order doesn't matter.
static REGISTRY: &[&dyn NativeRule] = &[
    &css_values::AlphaValueNotation,
    &css_declarations::AnnotationNoUnknown,
    &css_atrules::AtRuleDescriptorNoUnknown,
    &css_atrules::AtRuleDescriptorValueNoUnknown,
    &css_atrules::AtRuleNoDeprecated,
    &css_atrules::AtRuleNoUnknown,
    &css_atrules::AtRuleNoVendorPrefix,
    &css_atrules::AtRulePreludeNoInvalid,
    &css_selectors::BlockNoRedundantNestedStyleRules,
    &css_values::ColorFunctionAliasNotation,
    &css_values::ColorFunctionNotation,
    &css_values::ColorHexLength,
    &css_misc::CommentNoEmpty,
    &css_atrules::ContainerNamePattern,
    &css_atrules::CustomMediaPattern,
    &css_declarations::CustomPropertyNoMissingVarFunction,
    &css_declarations::CustomPropertyPattern,
    &css_declarations::DeclarationBlockNoDuplicateCustomProperties,
    &css_declarations::DeclarationBlockNoDuplicateProperties,
    &css_declarations::DeclarationBlockNoRedundantLonghandProperties,
    &css_declarations::DeclarationBlockNoShorthandPropertyOverrides,
    &css_declarations::DeclarationPropertyValueKeywordNoDeprecated,
    &css_declarations::DeclarationPropertyValueNoUnknown,
    &css_declarations::FontFamilyNoDuplicateNames,
    &css_declarations::FontFamilyNoMissingGenericFamilyKeyword,
    &css_values::FunctionCalcNoUnspacedOperator,
    &css_values::HueDegreeNotation,
    &css_atrules::ImportNotation,
    &css_atrules::KeyframeBlockNoDuplicateSelectors,
    &css_atrules::KeyframeDeclarationNoImportant,
    &css_atrules::KeyframeSelectorNotation,
    &css_atrules::KeyframesNamePattern,
    &css_atrules::LayerNamePattern,
    &css_values::LengthZeroNoUnit,
    &css_values::LightnessNotation,
    &css_media::MediaFeatureNameNoUnknown,
    &css_media::MediaFeatureNameNoVendorPrefix,
    &css_media::MediaFeatureNameValueNoUnknown,
    &css_media::MediaFeatureRangeNotation,
    &css_media::MediaQueryNoInvalid,
    &css_media::MediaTypeNoDeprecated,
    &css_declarations::NamedGridAreasNoInvalid,
    &css_selectors::NestingSelectorNoMissingScopingRoot,
    &css_selectors::NoDescendingSpecificity,
    &css_atrules::NoDuplicateAtImportRules,
    &css_selectors::NoDuplicateSelector,
    &css_misc::NoEmptyBlock,
    &css_misc::NoEmptySource,
    &css_declarations::NoImportant,
    &css_misc::NoInvalidDoubleSlashComments,
    &css_atrules::NoInvalidPositionAtImportRule,
    &css_declarations::NoInvalidPositionDeclaration,
    &css_misc::NoIrregularWhitespace,
    &css_declarations::NoUnknownProperty,
    &css_values::NumberMaxPrecision,
    &css_declarations::PropertyNoDeprecated,
    &css_declarations::PropertyNoVendorPrefix,
    &css_selectors::SelectorAnbNoUnmatchable,
    &css_selectors::SelectorClassPattern,
    &css_selectors::SelectorIdPattern,
    &css_selectors::SelectorNoVendorPrefix,
    &css_selectors::SelectorNotNotation,
    &css_selectors::SelectorPseudoClassNoUnknown,
    &css_selectors::SelectorPseudoElementColonNotation,
    &css_selectors::SelectorPseudoElementNoUnknown,
    &css_selectors::SelectorTypeNoUnknown,
    &css_declarations::ShorthandPropertyNoRedundantValues,
    &css_selectors::StringNoNewline,
    &css_atrules::SyntaxStringNoInvalid,
    &css_declarations::ValueNoVendorPrefix,
    &classes::ConstructorSuper,
    &accessors::GetterReturn,
    &conventions::ImportGroups,
    &expressions::NoConstantBinaryExpression,
    &duplicates::NoDupeArgs,
    &duplicates::NoDupeClassMembers,
    &duplicates::NoDupeElseIf,
    &duplicates::NoDupeKeys,
    &duplicates::NoDuplicateCase,
    &literals::NoEmptyCharacterClass,
    &literals::NoFallthrough,
    &regexp::NoInvalidRegexp,
    &literals::NoIrregularWhitespace,
    &literals::NoLossOfPrecision,
    &regexp::NoMisleadingCharacterClass,
    &literals::NoRegexSpaces,
    &accessors::NoSetterReturn,
    &literals::NoSparseArrays,
    &classes::NoThisBeforeSuper,
    &no_undef::NoUndef,
    &literals::NoUnexpectedMultiline,
    &no_unreachable::NoUnreachable,
    &expressions::NoUnsafeOptionalChaining,
    &classes::NoUnusedPrivateClassMembers,
    &JS_NO_UNUSED_VARS,
    &useless_assignment::NoUselessAssignment,
    &regexp::NoUselessBackreference,
    &useless_escape::NoUselessEscape,
    &typescript::NoVar,
    &typescript::PreferConst,
    &data_json::NoDuplicateKeys,
    &data_json::PackageEngines,
    &data_json::PackageScriptsSorted,
    &data_json::TsconfigStrict,
    &data_md::CommandsShowOutput,
    &data_md::DescriptiveLinkText,
    &data_md::FirstLineHeading,
    &data_md::HeadingIncrement,
    &data_md::LinkFragments,
    &data_md::LinkImageReferenceDefinitions,
    &data_md::NoBareUrls,
    &data_md::NoDuplicateHeading,
    &data_md::NoEmphasisAsHeading,
    &data_md::NoInlineHtml,
    &data_md::NoReversedLinks,
    &data_md::ReferenceLinksImages,
    &data_md::SingleH1,
    &data_md::TableColumnCount,
    &node::NoDeprecatedApi,
    &react::ExhaustiveDeps,
    &react::JsxNoDuplicateProps,
    &react::NoUnknownProperty,
    &conventions::PropsTypeName,
    &react::RequireRenderReturn,
    &react::RulesOfHooks,
    &stencil::BanDefaultTrue,
    &stencil::BanExportedConstEnums,
    &stencil::BanPrefix,
    &stencil::DecoratorsContext,
    &stencil::HostDataDeprecated,
    &stencil::MethodsMustBePublic,
    &stencil::NoUnusedWatch,
    &stencil::OwnMethodsMustBePrivate,
    &stencil::OwnPropsMustBePrivate,
    &stencil::PreferVdomListener,
    &stencil::PropReadonly,
    &stencil::PropsMustBePublic,
    &stencil::RequiredJsdoc,
    &stencil::ReservedMemberNames,
    &stencil::StrictMutable,
    &stencil::TagPrefix,
    &data_toml::DepsSorted,
    &data_toml::NoDuplicateKeys,
    &data_toml::PyprojectRequiresPython,
    &typescript::NoDuplicateEnumValues,
    &typescript::NoUnsafeDeclarationMerging,
    &TS_NO_UNUSED_EXPRESSIONS,
    &TS_NO_UNUSED_VARS,
    &typescript::NoWrapperObjectTypes,
    &typescript::PreferAsConst,
    &conventions::TypeOnlyImports,
    &data_yaml::DocumentStart,
    &data_yaml::NoAmbiguousBooleans,
    &data_yaml::NoAmbiguousNumbers,
    &data_yaml::NoDuplicateKeys,
    &data_yaml::NoUndeclaredAlias,
];

/// Every registered native rule.
pub fn registry() -> &'static [&'static dyn NativeRule] {
    REGISTRY
}

pub fn lookup(id: &str) -> Option<&'static dyn NativeRule> {
    REGISTRY.iter().copied().find(|r| r.id() == id)
}

#[cfg(test)]
pub(crate) fn run(rule: &dyn NativeRule, language: Language, src: &str) -> Vec<NativeFinding> {
    let root = codelaw_lang::parse(language, src);
    let model = codelaw_semantic::build(language, &root, src);
    let options = BTreeMap::new();
    let data = BTreeMap::new();
    let ctx = NativeContext { source: src, language, semantic: model.as_ref(), options: &options, data: &data };
    let mut out = Vec::new();
    for node in root.root().dfs() {
        if rule.kinds().contains(&node.kind().as_ref()) {
            rule.check(node, &ctx, &mut out);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn ids_are_unique() {
        let ids: Vec<&str> = super::registry().iter().map(|r| r.id()).collect();
        let unique: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "a native id is registered twice");
    }
}
