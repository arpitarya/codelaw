/**
 * @file CSS grammar for tree-sitter
 * @author Max Brunsfeld <maxbrunsfeld@gmail.com>
 * @author Amaan Qureshi <contact@amaanq.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const RANGE_OPERATORS = ['=', '<', '<=', '>', '>='];

export default grammar({
  name: 'css',

  extras: $ => [
    /\s/,
    $.comment,
    $.js_comment,
  ],

  externals: $ => [
    $._descendant_operator,
    $._pseudo_class_selector_colon,
    $.__error_recovery,
  ],

  inline: $ => [
    $._top_level_item,
    $._block_item,
  ],

  rules: {
    stylesheet: $ => repeat($._top_level_item),

    _top_level_item: $ => choice(
      $.declaration,
      $.rule_set,
      $.import_statement,
      $.media_statement,
      $.charset_statement,
      $.namespace_statement,
      $.keyframes_statement,
      $.supports_statement,
      $.scope_statement,
      $.at_rule,
    ),

    // Statements

    import_statement: $ => seq(
      '@import',
      $._value,
      optional($._import_layer),
      optional($._import_supports),
      sep(',', $._query),
      ';',
    ),

    // `layer` or `layer(name)` after the URL of an `@import`
    // (CSS Cascading and Inheritance Level 5). A bare `layer` keeps the
    // `keyword_query` node it has always parsed as.
    _import_layer: $ => choice(
      alias('layer', $.keyword_query),
      alias($._import_layer_function, $.call_expression),
    ),

    _import_layer_function: $ => seq(
      alias('layer', $.function_name),
      $.arguments,
    ),

    // `supports(…)` after the URL (and layer) of an `@import` (Cascade 5):
    // a condition, or a bare declaration such as `supports(display: grid)`.
    _import_supports: $ => alias($._import_supports_function, $.call_expression),

    _import_supports_function: $ => seq(
      alias('supports', $.function_name),
      alias($._import_supports_arguments, $.arguments),
    ),

    _import_supports_arguments: $ => seq(
      token.immediate('('),
      choice(
        $._query,
        alias($._supports_declaration, $.feature_query),
      ),
      ')',
    ),

    _supports_declaration: $ => prec.left(seq(
      alias($.identifier, $.feature_name),
      ':',
      repeat1($._value),
    )),

    media_statement: $ => seq(
      '@media',
      sep1(',', $._query),
      $.block,
    ),

    charset_statement: $ => seq(
      '@charset',
      $._value,
      ';',
    ),

    namespace_statement: $ => seq(
      '@namespace',
      optional(alias($.identifier, $.namespace_name)),
      choice($.string_value, $.call_expression),
      ';',
    ),

    keyframes_statement: $ => seq(
      choice(
        '@keyframes',
        alias(/@[-a-z]+keyframes/, $.at_keyword),
      ),
      alias($.identifier, $.keyframes_name),
      $.keyframe_block_list,
    ),

    keyframe_block_list: $ => seq(
      '{',
      repeat($.keyframe_block),
      '}',
    ),

    keyframe_block: $ => seq(
      sep1(',', choice($.from, $.to, $.integer_value, $.float_value, $.keyframe_range)),
      $.block,
    ),

    // A named timeline range and a percentage (Scroll-driven Animations):
    // `entry 0%`, `exit 100%`.
    keyframe_range: $ => seq(
      alias($.identifier, $.plain_value),
      choice($.integer_value, $.float_value),
    ),

    from: _ => 'from',
    to: _ => 'to',

    supports_statement: $ => seq(
      '@supports',
      $._query,
      $.block,
    ),

    scope_statement: $ => seq(
      '@scope',
      optional(
        seq(
          '(', $._selector, ')',
          optional(seq('to', '(', $._selector, ')')),
        ),
      ),
      $.block,
    ),

    postcss_statement: $ => prec(-1, seq(
      $.at_keyword,
      repeat(choice($._value, $.important_value)),
      ';',
    )),

    // Any other at-rule. Each comma-separated part of the prelude is one or
    // more queries, so a name may come before a query (`@container sidebar
    // (min-width: 400px)`, `@custom-media --small (max-width: 30em)`), a
    // family name may be several words (`@font-feature-values Font One`),
    // and a page selector may follow `@page` (`@page :first`, `@page wide:left`).
    at_rule: $ => seq(
      $.at_keyword,
      sep(',', repeat1(choice($._query, $.page_selector))),
      choice(';', $.block),
    ),

    page_selector: $ => prec.left(1, choice(
      seq(alias($.identifier, $.tag_name), repeat1($._page_pseudo)),
      seq(':', alias($.identifier, $.class_name), repeat($._page_pseudo)),
    )),

    _page_pseudo: $ => seq(token.immediate(':'), alias($.identifier, $.class_name)),

    // Rule sets

    rule_set: $ => seq(
      $.selectors,
      $.block,
    ),

    selectors: $ => sep1(',', $._selector),

    block: $ => seq(
      '{',
      repeat($._block_item),
      optional(alias($.last_declaration, $.declaration)),
      '}',
    ),

    _block_item: $ => choice(
      $.declaration,
      $.rule_set,
      $.import_statement,
      $.media_statement,
      $.charset_statement,
      $.namespace_statement,
      $.keyframes_statement,
      $.supports_statement,
      $.scope_statement,
      $.postcss_statement,
      $.at_rule,
    ),

    // Selectors

    _selector: $ => choice(
      $.universal_selector,
      alias($.identifier, $.tag_name),
      $.class_selector,
      $.nesting_selector,
      $.pseudo_class_selector,
      $.pseudo_element_selector,
      $.id_selector,
      $.attribute_selector,
      $.string_value,
      $.child_selector,
      $.descendant_selector,
      $.sibling_selector,
      $.adjacent_sibling_selector,
      $.namespace_selector,
    ),

    nesting_selector: _ => '&',

    universal_selector: _ => '*',

    class_selector: $ => prec(1, seq(
      optional($._selector),
      '.',
      $.class_name,
    )),

    pseudo_class_selector: $ => seq(
      optional($._selector),
      alias($._pseudo_class_selector_colon, ':'),
      choice(
        // Either a specific pseudo-class that can only accept a selector…
        seq(
          alias(
            choice('has', 'not', 'is', 'where', 'host', 'host-context'),
            $.class_name,
          ),
          alias($.pseudo_class_with_selector_arguments, $.arguments),
        ),

        // …or an `nth-child` or `nth-last-child` selector (which can
        // optionally accept a selector)…
        $._nth_child_pseudo_class_selector,

        // …or any other pseudo-class (for which we'll allow a more diverse set
        // of arguments).
        seq(
          $.class_name,
          optional(alias($.pseudo_class_arguments, $.arguments)),
        ),

        // …or a standalone `host` pseudo-class (as `:host` doesn't require arguments).
        alias('host', $.class_name),
      ),
    ),

    // Only `nth-child`/`nth-last-child`, not `nth-of-type`/`nth-last-of-type`,
    // allows an optional filtering selector as a parameter.
    _nth_child_pseudo_class_selector: $ => seq(
      alias(
        choice('nth-child', 'nth-last-child'),
        $.class_name,
      ),
      alias($.pseudo_class_nth_child_arguments, $.arguments),
    ),

    pseudo_element_selector: $ => seq(
      optional($._selector),
      '::',
      alias($.identifier, $.tag_name),
      optional(alias($.pseudo_element_arguments, $.arguments)),
    ),

    id_selector: $ => seq(
      optional($._selector),
      '#',
      alias($.identifier, $.id_name),
    ),

    attribute_selector: $ => seq(
      optional($._selector),
      token(prec(1, '[')),
      alias(choice($.identifier, $.namespace_selector), $.attribute_name),
      optional(seq(
        choice('=', '~=', '^=', '|=', '*=', '$='),
        $._value,
      )),
      ']',
    ),

    child_selector: $ => prec.left(seq(optional($._selector), '>', $._selector)),

    descendant_selector: $ => prec.left(seq($._selector, $._descendant_operator, $._selector)),

    sibling_selector: $ => prec.left(seq(optional($._selector), '~', $._selector)),

    adjacent_sibling_selector: $ => prec.left(seq(optional($._selector), '+', $._selector)),

    namespace_selector: $ => prec.left(seq(optional($._selector), '|', $._selector)),

    pseudo_class_arguments: $ => seq(
      token.immediate('('),
      sep(',', choice($._selector, repeat1($._value))),
      ')',
    ),

    pseudo_class_with_selector_arguments: $ => seq(
      token.immediate('('),
      sep(',', $._selector),
      ')',
    ),

    pseudo_class_nth_child_arguments: $ => prec(-1, seq(
      token.immediate('('),
      choice(
        alias('even', $.plain_value),
        alias('odd', $.plain_value),
        $.integer_value,
        alias($._nth_functional_notation, $.plain_value),
      ),
      optional(
        seq(
          'of',
          $._selector,
        ),
      ),
      ')',
    )),

    // An+B notation for `nth-child`/`nth-last-child`.
    _nth_functional_notation: _ => /-?(\d)*n\s*(\+\s*\d+)?/,

    pseudo_element_arguments: $ => seq(
      token.immediate('('),
      sep(',', choice($._selector, repeat1($._value))),
      ')',
    ),

    // Declarations

    declaration: $ => choice(
      seq(
        alias($.identifier, $.property_name),
        ':',
        $._value,
        repeat(seq(
          optional(','),
          $._value,
        )),
        optional(choice($.important, $.important_value)),
        ';',
      ),
      // A custom property whose value is a `{}` block (`--x: { color: red }`).
      seq(
        alias($.identifier, $.property_name),
        ':',
        $.block,
        optional(';'),
      ),
    ),

    last_declaration: $ => prec(1, seq(
      alias($.identifier, $.property_name),
      ':',
      $._value,
      repeat(seq(
        optional(','),
        $._value,
      )),
      optional(choice($.important, $.important_value)),
    )),

    important: _ => '!important',

    // Media queries

    _query: $ => choice(
      alias($.identifier, $.keyword_query),
      $.feature_query,
      $.range_query,
      $.binary_query,
      $.unary_query,
      $.selector_query,
      $.parenthesized_query,
    ),

    feature_query: $ => seq(
      '(',
      alias($.identifier, $.feature_name),
      ':',
      repeat1($._value),
      ')',
    ),

    // A range media feature (Media Queries Level 4): `(width >= 600px)`,
    // `(600px <= width)`, `(400px < width <= 700px)`.
    range_query: $ => seq(
      '(',
      choice(
        seq(alias($.identifier, $.feature_name), choice(...RANGE_OPERATORS), $._range_value),
        seq($._range_value, choice(...RANGE_OPERATORS), alias($.identifier, $.feature_name)),
        seq(
          $._range_value,
          choice('<', '<='),
          alias($.identifier, $.feature_name),
          choice('<', '<='),
          $._range_value,
        ),
        seq(
          $._range_value,
          choice('>', '>='),
          alias($.identifier, $.feature_name),
          choice('>', '>='),
          $._range_value,
        ),
      ),
      ')',
    ),

    _range_value: $ => choice(
      $.integer_value,
      $.float_value,
      $.call_expression,
      alias($._ratio, $.binary_expression),
    ),

    _ratio: $ => seq(
      choice($.integer_value, $.float_value),
      '/',
      choice($.integer_value, $.float_value),
    ),

    parenthesized_query: $ => seq(
      '(',
      $._query,
      ')',
    ),

    binary_query: $ => prec.left(seq(
      $._query,
      choice('and', 'or'),
      $._query,
    )),

    unary_query: $ => prec(1, seq(
      choice('not', 'only'),
      $._query,
    )),

    selector_query: $ => seq(
      'selector',
      '(',
      $._selector,
      ')',
    ),

    // Property Values

    _value: $ => prec(-1, choice(
      alias($.identifier, $.plain_value),
      $.plain_value,
      $.color_value,
      $.integer_value,
      $.float_value,
      $.string_value,
      $.grid_value,
      $.binary_expression,
      $.parenthesized_value,
      $.call_expression,
      $.important,
    )),

    parenthesized_value: $ => seq(
      '(',
      $._value,
      ')',
    ),

    color_value: _ => seq('#', token.immediate(/[0-9a-fA-F]{3,8}/)),

    string_value: $ => choice(
      seq(
        '\'',
        repeat(choice(
          alias(/[^\\'\n]+/, $.string_content),
          $.escape_sequence,
        )),
        '\'',
      ),
      seq(
        '"',
        repeat(choice(
          alias(/[^\\"\n]+/, $.string_content),
          $.escape_sequence,
        )),
        '"',
      ),
    ),

    escape_sequence: _ => token(seq(
      '\\',
      choice(
        /[0-9a-fA-F]{1,6}\s?/,
        /[^0-9a-fA-F\n\r]/,
      ),
    )),

    integer_value: $ => seq(
      token(seq(
        optional(choice('+', '-')),
        /\d+/,
      )),
      optional($.unit),
    ),

    float_value: $ => seq(
      token(seq(
        optional(choice('+', '-')),
        /\d*/,
        choice(
          seq('.', /\d+/),
          seq(/[eE]/, optional('-'), /\d+/),
          seq('.', /\d+/, /[eE]/, optional('-'), /\d+/),
        ),
      )),
      optional($.unit),
    ),

    unit: _ => token.immediate(/[a-zA-Z%]+/),

    grid_value: $ => seq(
      '[',
      sep1(',', $._value),
      ']',
    ),

    call_expression: $ => seq(
      alias($.identifier, $.function_name),
      $.arguments,
    ),

    binary_expression: $ => prec.left(seq(
      $._value,
      choice('+', '-', '*', '/'),
      $._value,
    )),

    arguments: $ => seq(
      token.immediate('('),
      sep(choice(',', ';'), repeat1($._value)),
      ')',
    ),

    class_name: $ => seq(
      choice($.identifier, $.escape_sequence),
      repeat(choice(
        alias(/[a-zA-Z0-9-_\xA0-\xFF]+/, $.identifier),
        $.escape_sequence,
      )),
    ),

    identifier: _ => /(--|-?[a-zA-Z_\xA0-\xFF])[a-zA-Z0-9-_\xA0-\xFF]*/,

    at_keyword: _ => /@[a-zA-Z-_]+/,

    js_comment: _ => token(prec(-1, seq('//', /.*/))),

    comment: _ => token(seq(
      '/*',
      /[^*]*\*+([^/*][^*]*\*+)*/,
      '/',
    )),

    plain_value: _ => token(seq(
      repeat(choice(
        /[-_]/,
        /\/[^\*\s,;!{}()\[\]]/, // Slash not followed by a '*' (which would be a comment)
      )),
      /[a-zA-Z]/,
      repeat(choice(
        /[^/\s,;!{}()\[\]]/, // Not a slash, not a delimiter character
        /\/[^\*\s,;!{}()\[\]]/, // Slash not followed by a '*' (which would be a comment)
      )),
    )),

    important_value: _ => token(seq(
      '!',
      /[a-zA-Z]/,
      repeat(/[a-zA-Z0-9-_]/),
    )),
  },
});

/**
 * Creates a rule to optionally match one or more of the rules separated by `separator`
 *
 * @param {RuleOrLiteral} separator
 *
 * @param {RuleOrLiteral} rule
 *
 * @returns {ChoiceRule}
 */
function sep(separator, rule) {
  return optional(sep1(separator, rule));
}

/**
 * Creates a rule to match one or more of the rules separated by `separator`
 *
 * @param {RuleOrLiteral} separator
 *
 * @param {RuleOrLiteral} rule
 *
 * @returns {SeqRule}
 */
function sep1(separator, rule) {
  return seq(rule, repeat(seq(separator, rule)));
}
