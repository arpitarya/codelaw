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
mod duplicates;
mod expressions;
mod literals;
mod no_undef;
mod no_unreachable;
mod node;
mod python_bugbear;
mod python_conventions;
mod python_format;
mod python_isort;
mod python_pycodestyle;
mod python_pyflakes;
mod python_util;
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

/// Sorted by id.
static REGISTRY: &[&dyn NativeRule] = &[
    &classes::ConstructorSuper,
    &accessors::GetterReturn,
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
    &node::NoDeprecatedApi,
    &python_bugbear::AbstractBaseClassWithoutAbstractMethod,
    &python_isort::AllSorted,
    &python_pycodestyle::AmbiguousClassName,
    &python_pycodestyle::AmbiguousFunctionName,
    &python_pycodestyle::AmbiguousVariableName,
    &python_bugbear::AssertFalse,
    &python_bugbear::AssertRaisesException,
    &python_pyflakes::AssertTuple,
    &python_bugbear::AssignmentToOsEnviron,
    &python_bugbear::BatchedWithoutExplicitStrict,
    &python_pyflakes::BreakOutsideLoop,
    &python_bugbear::CachedInstanceMethod,
    &python_pyflakes::ContinueOutsideLoop,
    &python_pyflakes::DefaultExceptNotLast,
    &python_pyflakes::MultiValueRepeatedKeyLiteral,
    &python_pyflakes::MultiValueRepeatedKeyVariable,
    &python_bugbear::DuplicateHandlerException,
    &python_bugbear::DuplicateValue,
    &python_bugbear::DuplicateTryBlockException,
    &python_bugbear::EmptyMethodWithoutAbstractDecorator,
    &python_bugbear::ExceptWithEmptyTuple,
    &python_bugbear::ExceptWithNonExceptionClasses,
    &python_bugbear::FStringDocstring,
    &python_pyflakes::FStringMissingPlaceholders,
    &python_format::StringDotFormatInvalidFormat,
    &python_format::StringDotFormatMissingArguments,
    &python_format::StringDotFormatMixingAutomatic,
    &python_format::StringDotFormatExtraNamedArguments,
    &python_format::StringDotFormatExtraPositionalArguments,
    &python_pyflakes::ForwardAnnotationSyntaxError,
    &python_bugbear::FunctionCallInDefaultArgument,
    &python_bugbear::FunctionUsesLoopVariable,
    &python_bugbear::GetAttrWithConstant,
    &python_pyflakes::IfTuple,
    &python_pyflakes::ImportShadowedByLoopVar,
    &python_pyflakes::InvalidPrintSyntax,
    &python_pyflakes::IsLiteral,
    &python_bugbear::JumpStatementInFinally,
    &python_pycodestyle::LambdaAssignment,
    &python_pyflakes::LateFutureImport,
    &python_bugbear::LoopVariableOverridesIterator,
    &python_bugbear::MapWithoutExplicitStrict,
    &python_pycodestyle::ModuleImportNotAtTopOfFile,
    &python_pycodestyle::MultipleImportsOnOneLine,
    &python_pyflakes::MultipleStarredExpressions,
    &python_bugbear::MutableContextvarDefault,
    &python_pyflakes::NestedStarImport,
    &python_pycodestyle::BareExcept,
    &python_bugbear::NoExplicitStacklevel,
    &python_bugbear::MutableArgumentDefault,
    &python_conventions::NoPrint,
    &python_pyflakes::UndefinedLocalWithImportStar,
    &python_pycodestyle::NoneComparison,
    &python_pycodestyle::NotInTest,
    &python_pycodestyle::NotIsTest,
    &python_format::PercentFormatPositionalCountMismatch,
    &python_format::PercentFormatExpectedMapping,
    &python_format::PercentFormatExpectedSequence,
    &python_format::PercentFormatInvalidFormat,
    &python_format::PercentFormatMissingArgument,
    &python_format::PercentFormatMixedPositionalAndNamed,
    &python_format::PercentFormatStarRequiresSequence,
    &python_format::PercentFormatUnsupportedFormatCharacter,
    &python_format::PercentFormatExtraNamedArguments,
    &python_bugbear::RaiseLiteral,
    &python_pyflakes::RaiseNotImplemented,
    &python_bugbear::RaiseWithoutFromInsideExcept,
    &python_bugbear::ReSubPositionalArgs,
    &python_pyflakes::RedefinedWhileUnused,
    &python_bugbear::RedundantTupleInExceptionHandler,
    &python_pyflakes::ReturnOutsideFunction,
    &python_bugbear::ReuseOfGroupbyGenerator,
    &python_bugbear::SetAttrWithConstant,
    &python_bugbear::StarArgUnpackingAfterKeywordArg,
    &python_pyflakes::StarImportUsage,
    &python_bugbear::StaticKeyDictComprehension,
    &python_bugbear::StripWithMultiCharacters,
    &python_pyflakes::ExpressionsInStarAssignment,
    &python_pycodestyle::TrueFalseComparison,
    &python_pycodestyle::TypeComparison,
    &python_bugbear::UnaryPrefixIncrementDecrement,
    &python_pyflakes::UndefinedExport,
    &python_pyflakes::FutureFeatureNotDefined,
    &python_pyflakes::UndefinedLocal,
    &python_pyflakes::UndefinedName,
    &python_bugbear::UnintentionalTypeAnnotation,
    &python_bugbear::UnreliableCallableCheck,
    &python_isort::UnsortedImports,
    &python_pyflakes::UnusedAnnotation,
    &python_pyflakes::UnusedImport,
    &python_bugbear::UnusedLoopControlVariable,
    &python_pyflakes::UnusedVariable,
    &python_bugbear::UselessComparison,
    &python_bugbear::UselessContextlibSuppress,
    &python_bugbear::UselessExpression,
    &python_pyflakes::YieldOutsideFunction,
    &python_bugbear::ZipWithoutExplicitStrict,
    &react::ExhaustiveDeps,
    &react::JsxNoDuplicateProps,
    &react::NoUnknownProperty,
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
    &typescript::NoDuplicateEnumValues,
    &typescript::NoUnsafeDeclarationMerging,
    &TS_NO_UNUSED_EXPRESSIONS,
    &TS_NO_UNUSED_VARS,
    &typescript::NoWrapperObjectTypes,
    &typescript::PreferAsConst,
];

/// Every registered native rule, sorted by id.
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
    fn ids_are_unique_and_sorted() {
        let ids: Vec<&str> = super::registry().iter().map(|r| r.id()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(ids, sorted);
    }
}
