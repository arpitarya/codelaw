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
mod css_modules;
mod css_selectors;
mod css_syntax;
mod css_values;
mod scss_rules;
mod data_json;
mod data_md;
mod data_toml;
mod data_yaml;
pub mod deps;
mod duplicates;
mod expressions;
#[cfg(feature = "fault-injection")]
mod fault;
mod html_a11y;
mod html_content;
mod html_document;
mod html_dom;
mod html_rules;
mod html_security;
mod js_ast;
mod js_console;
mod size;
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
mod react_hooks;
mod tanstack_query;
mod redux_toolkit;
mod size_lines;
mod regexp;
mod stencil;
mod typescript;
mod unused_vars;
mod useless_assignment;
mod useless_escape;
mod util;

use codelaw_core::{Language, Severity, TextEdit};
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
    /// The file itself: its path, its raw bytes and a read-only view of the
    /// files around it (decision 15b). For an embedded region, the host file.
    pub file: &'a dyn FileContext,
}

/// The file a native rule checks, beyond its text (SR-ARCHITECTURE decision
/// 15b): where it is, its bytes as they are on disk, and a read-only view of
/// the other files of the project.
///
/// Paths are relative to the project root, `/`-separated, with no `.` or
/// `..` segment ([`resolve_path`] makes one from a link). Nothing outside the
/// project root can be named, and nothing can be written. The engine records
/// every question a rule asks, so a cached result is reused only while every
/// answer still holds.
pub trait FileContext: Sync {
    /// The file's path.
    fn path(&self) -> &str;
    /// The file's bytes, a UTF-8 byte-order mark included: the text rules
    /// see has none.
    fn bytes(&self) -> &[u8];
    /// Whether a file or folder exists at `path`.
    fn exists(&self, path: &str) -> bool;
    /// Whether `path` is a folder.
    fn is_dir(&self, path: &str) -> bool;
    /// The text of the file at `path`, when it exists and is UTF-8.
    fn read(&self, path: &str) -> Option<String>;
    /// The names in the folder at `path` (`""` is the root), sorted.
    fn list(&self, path: &str) -> Option<Vec<String>>;
    /// The as-of date, `YYYY-MM-DD`, when there is one (L5 decision 2):
    /// the only date a rule may judge by. The engine records the question,
    /// so a cached result is reused only on the same date.
    fn as_of(&self) -> Option<String> {
        None
    }
}

/// A file with no neighbours: what a rule sees when only its text is known.
#[derive(Clone, Debug, Default)]
pub struct DetachedFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

impl DetachedFile {
    pub fn new(path: impl Into<String>, source: &str) -> Self {
        DetachedFile { path: path.into(), bytes: source.as_bytes().to_vec() }
    }
}

impl FileContext for DetachedFile {
    fn path(&self) -> &str {
        &self.path
    }
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn exists(&self, path: &str) -> bool {
        path.is_empty() || path == self.path
    }
    fn is_dir(&self, path: &str) -> bool {
        path.is_empty()
    }
    fn read(&self, path: &str) -> Option<String> {
        (path == self.path).then(|| String::from_utf8_lossy(&self.bytes).into_owned())
    }
    fn list(&self, path: &str) -> Option<Vec<String>> {
        path.is_empty().then(|| vec![self.path.clone()])
    }
}

/// Resolve `link` against the folder of `from` (both project-relative): the
/// project-relative path it names, or `None` when it leaves the project
/// root. A `link` starting with `/` is relative to the root.
pub fn resolve_path(from: &str, link: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    if !link.starts_with('/') {
        parts.extend(from.split('/').filter(|p| !p.is_empty()));
        parts.pop();
    }
    for seg in link.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
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
    /// A config section whose keys the rule reads as its options (the
    /// rule's own options win): `[deps]` for the dependency rules
    /// (SR-RULE-PACKS decision 15).
    fn config_section(&self) -> Option<&'static str> {
        None
    }
    /// The severity of one finding, for a rule that grades its findings;
    /// `None` keeps the rule's.
    /// Structured data about one finding, for `--format json` and SARIF
    /// (the measured value and the limit, for a size rule); `None` carries
    /// none.
    fn data_of(&self, _finding: &NativeFinding, _options: &BTreeMap<String, serde_json::Value>) -> Option<serde_json::Value> {
        None
    }
    fn severity_of(&self, _finding: &NativeFinding, _options: &BTreeMap<String, serde_json::Value>) -> Option<Severity> {
        None
    }
}

const JS_ONLY: &[Language] = &[Language::Javascript, Language::Jsx];
const TS_ONLY: &[Language] = &[Language::Typescript, Language::Tsx];

static JS_MAX_LINES: size::Size = size::Size::js("js/max-lines", size::Measure::FileLines);
static JS_MAX_LINES_PER_FUNCTION: size::Size = size::Size::js("js/max-lines-per-function", size::Measure::FunctionLines);
static JS_MAX_PARAMS: size::Size = size::Size::js("js/max-params", size::Measure::Params);
static PY_MAX_LINES: size::Size = size::Size::py("py/max-lines", size::Measure::FileLines);
static PY_MAX_LINES_PER_FUNCTION: size::Size = size::Size::py("py/max-lines-per-function", size::Measure::FunctionLines);
static PY_MAX_PARAMS: size::Size = size::Size::py("py/max-params", size::Measure::Params);
static JS_NO_UNUSED_VARS: unused_vars::NoUnusedVars = unused_vars::NoUnusedVars { id: "js/no-unused-vars", languages: JS_ONLY };
static TS_NO_UNUSED_VARS: unused_vars::NoUnusedVars = unused_vars::NoUnusedVars { id: "ts/no-unused-vars", languages: TS_ONLY };
static TS_NO_UNUSED_EXPRESSIONS: typescript::NoUnusedExpressions = typescript::NoUnusedExpressions { id: "ts/no-unused-expressions", languages: TS_ONLY };

static CSS_MAX_LINES: size_lines::MaxLines = size_lines::MaxLines::css("css/max-lines");
static HTML_MAX_LINES: size_lines::MaxLines = size_lines::MaxLines::html("html/max-lines");

/// Every native rule; ids are unique, order doesn't matter.
static REGISTRY: &[&dyn NativeRule] = &[
    &css_modules::ClassNaming,
    &css_modules::ComposesValid,
    &css_modules::NoGlobal,
    &scss_rules::AtExtendNoMissingPlaceholder,
    &scss_rules::AtIfNoNull,
    &scss_rules::AtMixinPattern,
    &scss_rules::AtRuleNoUnknown,
    &scss_rules::DollarVariablePattern,
    &scss_rules::DoubleSlashCommentEmpty,
    &scss_rules::FunctionNoUnknown,
    &scss_rules::LoadNoPartialLeadingUnderscore,
    &scss_rules::MaxNestingDepth,
    &scss_rules::NoDuplicateDollarVariables,
    &scss_rules::NoDuplicateMixins,
    &scss_rules::NoGlobalFunctionNames,
    &scss_rules::NoImport,
    &scss_rules::NoSlashDivision,
    &scss_rules::PercentPlaceholderPattern,
    &CSS_MAX_LINES,
    &HTML_MAX_LINES,
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
    &html_a11y::AreaAlt,
    &html_a11y::AriaHiddenBody,
    &html_a11y::AriaLabelMisuse,
    &html_content::AttributeAllowedValues,
    &html_content::AttributeMisuse,
    &html_a11y::AutocompletePassword,
    &html_rules::ButtonType,
    &html_content::CloseAttr,
    &html_content::CloseOrder,
    &html_content::Deprecated,
    &html_rules::DoctypeHtml,
    &html_content::ElementName,
    &html_content::ElementPermittedContent,
    &html_content::ElementPermittedOccurrences,
    &html_content::ElementPermittedOrder,
    &html_content::ElementPermittedParent,
    &html_content::ElementRequiredAncestor,
    &html_content::ElementRequiredAttributes,
    &html_content::ElementRequiredContent,
    &html_a11y::EmptyHeading,
    &html_a11y::EmptyTitle,
    &html_a11y::FieldsetLegend,
    &html_rules::FormDupName,
    &html_a11y::FormSubmit,
    &html_document::HeadingLevel,
    &html_a11y::HiddenFocusable,
    &html_a11y::ImgAlt,
    &html_a11y::ImgTitle,
    &html_content::InputAttributes,
    &html_a11y::InputImageAlt,
    &html_document::InputMissingLabel,
    &html_a11y::LinkText,
    &html_rules::LongTitle,
    &html_rules::MapDupName,
    &html_rules::MapIdName,
    &html_a11y::MetaRefresh,
    &html_document::MissingDoctype,
    &html_a11y::MultipleLabeledControls,
    &html_a11y::NoAbstractRole,
    &html_a11y::NoAutoplay,
    &html_rules::NoConditionalComment,
    &html_content::NoDeprecatedAttr,
    &html_rules::NoDupAttr,
    &html_rules::NoDupClass,
    &html_rules::NoDuplicateId,
    &html_content::NoImplicitClose,
    &html_rules::NoImplicitInputType,
    &html_rules::NoInlineScript,
    &html_rules::NoInlineStyle,
    &html_document::NoMissingReferences,
    &html_document::NoUtf8Bom,
    &html_rules::NoMultipleMain,
    &html_rules::NoRawCharacters,
    &html_a11y::NoRedundantAriaLabel,
    &html_a11y::NoRedundantFor,
    &html_a11y::NoRedundantRole,
    &html_rules::PreferButton,
    &html_a11y::PreferNativeElement,
    &html_rules::PreferTbody,
    &html_document::RequireSri,
    &html_content::ScriptElement,
    &html_rules::ScriptType,
    &html_rules::TargetBlankRel,
    &html_rules::TelNonBreaking,
    &html_a11y::TextContent,
    &html_a11y::ThScope,
    &html_a11y::UniqueLandmark,
    &html_rules::UnrecognizedCharRef,
    &html_a11y::ValidAutocomplete,
    &html_rules::ValidFor,
    &html_rules::ValidId,
    &html_content::VoidContent,
    &html_security::AngularjsUrlExpression,
    &html_security::CspMetaUnsafe,
    &html_security::FormCsrfToken,
    &html_security::IframeAllowPowerful,
    &html_security::IframeSandbox,
    &html_security::NoCredentialsInUrl,
    &html_security::NoExternalBase,
    &html_security::NoInlineEventHandler,
    &html_security::NoJavascriptUrl,
    &html_security::NoMixedContent,
    &html_security::NoPluginEmbed,
    &html_security::NoSecretInHiddenInput,
    &html_security::NoSensitiveComments,
    &html_security::NoSrcdocScript,
    &html_security::NoTemplateHtmlSink,
    &html_security::NoUnsafeTemplateInterpolation,
    &html_security::PasswordFormPost,
    &html_security::ReferrerPolicy,
    &html_security::SriCrossorigin,
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
    &js_console::NoConsole,
    &JS_MAX_LINES,
    &JS_MAX_LINES_PER_FUNCTION,
    &JS_MAX_PARAMS,
    &PY_MAX_LINES,
    &PY_MAX_LINES_PER_FUNCTION,
    &PY_MAX_PARAMS,
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
    &data_md::RelativeLinks,
    &data_md::SingleH1,
    &data_md::TableColumnCount,
    &node::Hashbang,
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
    &react::NoContext,
    &react_hooks::ErrorBoundaries,
    &react_hooks::Globals,
    &react_hooks::NoNestedComponents,
    &react_hooks::NoRefInRender,
    &react_hooks::NoSetStateInEffect,
    &react_hooks::NoSetStateInRender,
    &react_hooks::NoUnnecessaryUsePrefix,
    &react_hooks::PreferLazyStateInit,
    &react_hooks::Purity,
    &react_hooks::UseMemo,
    &react::JsxNoDuplicateProps,
    &react::NoUnknownProperty,
    &conventions::PropsTypeName,
    &react::RequireRenderReturn,
    &react::RulesOfHooks,
    &tanstack_query::ArrayQueryKey,
    &tanstack_query::ExhaustiveDeps,
    &tanstack_query::InfiniteQueryPropertyOrder,
    &tanstack_query::MutationPropertyOrder,
    &tanstack_query::NoMutatingSetQueryData,
    &tanstack_query::NoRemovedV4Options,
    &tanstack_query::NoRenamedV4Options,
    &tanstack_query::NoRestDestructuring,
    &tanstack_query::NoUnstableDeps,
    &tanstack_query::NoVoidQueryFn,
    &tanstack_query::NoUnsupportedSuspenseOptions,
    &tanstack_query::ObjectSyntax,
    &tanstack_query::StableQueryClient,
    &redux_toolkit::NoBlindPayloadReturn,
    &redux_toolkit::NoMutatingReducer,
    &redux_toolkit::NoNonSerializableState,
    &redux_toolkit::NoObjectSelector,
    &redux_toolkit::NoSequentialDispatch,
    &redux_toolkit::NoSideEffectsInReducer,
    &redux_toolkit::ObjectShorthandMapDispatch,
    &redux_toolkit::PreferCreateSlice,
    &redux_toolkit::SliceKeyNames,
    &stencil::BanDefaultTrue,
    &stencil::BanExportedConstEnums,
    &stencil::BanPrefix,
    &stencil::BanSideEffects,
    &stencil::DecoratorsContext,
    &stencil::EnforceSlotJsdoc,
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
    &deps::OutdatedDependency,
    &deps::StaleDepsSnapshot,
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
    #[cfg(feature = "fault-injection")]
    if let Some(r) = fault::lookup(id) {
        return Some(r);
    }
    REGISTRY.iter().copied().find(|r| r.id() == id)
}

/// A file and a fixed set of neighbours, for the tests of rules that read
/// them.
#[cfg(test)]
pub(crate) struct MemoryFiles {
    path: String,
    bytes: Vec<u8>,
    files: Vec<(String, String)>,
}

#[cfg(test)]
impl MemoryFiles {
    pub fn new(path: &str, source: &str, files: &[(&str, &str)]) -> Self {
        MemoryFiles { path: path.into(), bytes: source.as_bytes().to_vec(), files: files.iter().map(|(p, t)| (p.to_string(), t.to_string())).collect() }
    }
}

#[cfg(test)]
impl FileContext for MemoryFiles {
    fn path(&self) -> &str {
        &self.path
    }
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn exists(&self, path: &str) -> bool {
        path.is_empty() || path == self.path || self.is_dir(path) || self.files.iter().any(|(p, _)| p == path)
    }
    fn is_dir(&self, path: &str) -> bool {
        let dir = format!("{path}/");
        path.is_empty() || self.path.starts_with(&dir) || self.files.iter().any(|(p, _)| p.starts_with(&dir))
    }
    fn read(&self, path: &str) -> Option<String> {
        self.files.iter().find(|(p, _)| p == path).map(|(_, t)| t.clone())
    }
    fn list(&self, _: &str) -> Option<Vec<String>> {
        None
    }
}

#[cfg(test)]
pub(crate) fn run(rule: &dyn NativeRule, language: Language, src: &str) -> Vec<NativeFinding> {
    let root = codelaw_lang::parse(language, src);
    let model = codelaw_semantic::build(language, &root, src);
    let options = BTreeMap::new();
    let data = BTreeMap::new();
    let file = DetachedFile::new(format!("example.{}", language.extension()), src);
    let ctx = NativeContext { source: src, language, semantic: model.as_ref(), options: &options, data: &data, file: &file };
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

    #[test]
    fn resolve_path() {
        use super::resolve_path as r;
        assert_eq!(r("docs/a.md", "b.md").as_deref(), Some("docs/b.md"));
        assert_eq!(r("docs/a.md", "./x/../c.md").as_deref(), Some("docs/c.md"));
        assert_eq!(r("docs/a.md", "../README.md").as_deref(), Some("README.md"));
        assert_eq!(r("docs/a.md", "/src/x.rs").as_deref(), Some("src/x.rs"));
        assert_eq!(r("a.md", "../outside.md"), None);
        assert_eq!(r("a.md", "sub/").as_deref(), Some("sub"));
    }
}
