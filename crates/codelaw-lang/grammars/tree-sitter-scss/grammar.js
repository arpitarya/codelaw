/**
 * @file SCSS grammar for tree-sitter
 * @author Amaan Qureshi <amaanq12@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// codelaw: built on codelaw's in-tree tree-sitter-css fork (SR-ARCHITECTURE
// decision 3), not on the npm tree-sitter-css 0.20 upstream builds against.
const CSS = require('../tree-sitter-css/grammar.js').default;

module.exports = grammar(CSS, {
  name: 'scss',

  externals: ($, original) => original.concat([
    $._concat,
  ]),

  rules: {
    _top_level_item: ($, original) => choice(
      original,
      $.postcss_statement,
      $.use_statement,
      $.forward_statement,
      $.mixin_statement,
      $.include_statement,
      $.function_statement,
      $.return_statement,
      $.extend_statement,
      $.error_statement,
      $.warn_statement,
      $.debug_statement,
      $.at_root_statement,
      $.if_statement,
      $.each_statement,
      $.for_statement,
      $.while_statement,
    ),

    _block_item: ($, original) => choice(
      original,
      $.content_statement,
      $.mixin_statement,
      $.include_statement,
      $.function_statement,
      $.return_statement,
      $.extend_statement,
      $.error_statement,
      $.warn_statement,
      $.debug_statement,
      $.at_root_statement,
      $.if_statement,
      $.each_statement,
      $.for_statement,
      $.while_statement,
    ),

    // Selectors

    _selector: ($, original) => choice(
      original,
      alias($._concatenated_identifier, $.tag_name),
      $.placeholder,
    ),

    class_selector: $ => prec(1, seq(
      optional($._selector),
      choice('.', $.nesting_selector),
      choice($.class_name, alias($._concatenated_identifier, $.class_name)),
    )),

    // The CSS fork's pseudo-classes (`:nth-child(n + 3)`, `:has(…)`), plus an
    // interpolated name (codelaw).
    pseudo_class_selector: ($, original) => choice(
      original,
      seq(
        optional($._selector),
        alias($._pseudo_class_selector_colon, ':'),
        alias($._concatenated_identifier, $.class_name),
        optional(alias($.pseudo_class_arguments, $.arguments)),
      ),
    ),

    // `:nth-child(#{$n})`
    pseudo_class_nth_child_arguments: ($, original) => choice(
      original,
      seq(token.immediate('('), $.interpolation, ')'),
    ),

    pseudo_element_selector: $ => seq(
      optional($._selector),
      '::',
      alias(choice($.identifier, $._concatenated_identifier), $.tag_name),
      optional(alias($.pseudo_element_arguments, $.arguments)),
    ),

    // A colon ends a plain value when whitespace follows it, so a map key
    // (`(a: 1)`) is not read as `a:`; `http://…` stays one value (codelaw).
    plain_value: _ => token(seq(
      repeat(choice(
        /[-_]/,
        /\/[^\*\s,;!{}()\[\]]/,
      )),
      /[a-zA-Z]/,
      repeat(choice(
        /[^/:#\s,;!{}()\[\]]/,
        /:[^\s,;!{}()\[\]]/,
        /#[^\s,;!{}()\[\]]/,
        /\/[^\*\s,;!{}()\[\]]/,
      )),
    )),

    // `100%` is a unit, not the modulo operator (codelaw).
    unit: _ => token.immediate(prec(1, /[a-zA-Z%]+/)),

    // Declarations

    // A nested property (`margin: { top: 1px; }`, `font: 12px { … }`) or a
    // custom property holding a block ends in a block (codelaw).
    declaration: $ => seq(
      alias(
        choice($.identifier, $.variable, $._concatenated_identifier),
        $.property_name,
      ),
      ':',
      choice(
        seq(
          $._value,
          repeat(seq(optional(','), $._value)),
          repeat(choice($.important, $.important_value)),
          choice(';', seq($.block, optional(';'))),
        ),
        seq($.block, optional(';')),
      ),
    ),

    // Media queries

    _query: ($, original) => choice(
      original,
      prec(-1, $.interpolation),
    ),

    // Property Values

    _value: ($, original) => choice(
      original,
      prec(-1, choice(
        $.nesting_selector,
        $._concatenated_identifier,
        $.list_value,
        $.map_value,
      )),
      $.variable,
      alias($._signed_variable, $.variable),
      alias($._signed_interpolation, $.interpolation),
      $.unary_expression,
    ),

    // Media feature values can be Sass variables and interpolation; a function
    // in a range is a plain CSS one (`calc(…)`).
    _range_value: $ => choice(
      $.integer_value,
      $.float_value,
      prec(-1, $.variable),
      prec(-1, $.interpolation),
      alias($._range_call, $.call_expression),
      alias($._ratio, $.binary_expression),
    ),

    _range_call: $ => prec(-1, seq(alias($.identifier, $.function_name), $.arguments)),

    use_statement: $ => seq(
      '@use',
      $._value,
      optional($.as_clause),
      optional($.with_clause),
      ';',
    ),

    forward_statement: $ => seq(
      '@forward',
      $._value,
      optional($.as_clause),
      optional(choice($.show_clause, $.hide_clause)),
      optional($.with_clause),
      ';',
    ),

    // `as *`, `as name`, and @forward's prefix `as btn-*`.
    as_clause: $ => seq(
      'as',
      choice(
        '*',
        seq(alias($.identifier, $.module_name), optional(token.immediate('*'))),
      ),
    ),

    show_clause: $ => seq('show', sep1(',', choice($.identifier, $.variable))),

    hide_clause: $ => seq('hide', sep1(',', choice($.identifier, $.variable))),

    with_clause: $ => seq('with', $.map_value),

    mixin_statement: $ => seq(
      '@mixin',
      field('name', $.identifier),
      optional($.parameters),
      $.block,
    ),

    include_statement: $ => seq(
      '@include',
      choice($.identifier, alias($._module_member, $.identifier)),
      optional(alias($._include_arguments, $.arguments)),
      optional(seq('using', $.parameters)),
      choice($.block, ';'),
    ),

    // A namespaced member: `bp.media-up`.
    _module_member: _ => token(seq(/[a-zA-Z_-][a-zA-Z0-9_-]*/, '.', /[a-zA-Z_-][a-zA-Z0-9_-]*/)),

    _include_arguments: $ => seq(
      '(',
      optional(seq(
        sep1(',', alias($._include_argument, $.argument)),
        optional(','),
      )),
      ')',
    ),

    _include_argument: $ => seq(
      optional(seq(field('name', $.variable), ':')),
      field('value', $._value),
      repeat($._value),
      optional('...'),
    ),

    function_statement: $ => seq(
      '@function',
      field('name', $.identifier),
      optional($.parameters),
      $.block,
    ),

    parameters: $ => seq(
      '(',
      optional(seq(sep1(',', $.parameter), optional(','))),
      ')',
    ),

    parameter: $ => seq(
      $.variable,
      optional(choice(
        seq(':', field('default', $._value), repeat($._value)),
        '...',
      )),
    ),

    return_statement: $ => seq(
      '@return',
      sep1(',', seq($._value, repeat($._value))),
      ';',
    ),

    content_statement: $ => seq(
      '@content',
      optional(alias($._include_arguments, $.arguments)),
      ';',
    ),

    // `@import "a", "b";` — a list of Sass imports.
    import_statement: ($, original) => choice(
      original,
      seq('@import', $.string_value, repeat1(seq(',', $.string_value)), ';'),
    ),

    extend_statement: $ => seq(
      '@extend',
      sep1(",", choice($._value, $._selector)),
      optional($.important_value),
      ';',
    ),

    error_statement: $ => seq('@error', $._value, ';'),

    warn_statement: $ => seq('@warn', $._value, ';'),

    debug_statement: $ => seq('@debug', $._value, ';'),

    at_root_statement: $ => seq(
      '@at-root',
      optional(choice($.selectors, $.parenthesized_query)),
      $.block,
    ),

    if_statement: $ => seq(
      '@if',
      field('condition', $._value),
      $.block,
      repeat($.else_if_clause),
      optional($.else_clause),
    ),

    else_if_clause: $ => seq(
      '@else',
      'if',
      field('condition', $._value),
      $.block,
    ),

    else_clause: $ => seq('@else', $.block),

    each_statement: $ => seq(
      '@each',
      optional(seq(field('key', $.variable), ',')),
      field('value', $.variable),
      'in',
      $._value,
      $.block,
    ),

    for_statement: $ => seq(
      '@for',
      $.variable,
      'from',
      field('from', $._value),
      choice('through', 'to'),
      field('through', $._value),
      $.block,
    ),

    while_statement: $ => seq('@while', $._value, $.block),

    call_expression: $ => seq(
      alias(choice($.identifier, $.plain_value), $.function_name),
      $.arguments,
    ),

    binary_expression: $ => choice(
      prec.left(1, seq($._value, choice('and', 'or'), $._value)),
      prec.left(2, seq($._value, choice('==', '!=', '<', '>', '<=', '>='), $._value)),
      prec.left(3, seq($._value, choice('+', '-'), $._value)),
      prec.left(4, seq($._value, choice('*', '/', '%'), $._value)),
    ),

    unary_expression: $ => prec(5, seq('not', $._value)),

    // `-#{$x}`: a sign joined to an interpolation.
    _signed_interpolation: $ => seq(token(seq(/[-+]/, '#{')), $._value, repeat($._value), '}'),

    list_value: $ => choice(
      seq('(', ')'),
      seq('(', sep2(',', repeat1($._value)), optional(','), ')'),
    ),

    // A Sass map: `(key: value, …)`.
    map_value: $ => seq(
      '(',
      sep1(',', $.map_pair),
      optional(','),
      ')',
    ),

    map_pair: $ => seq(
      field('key', $._value),
      ':',
      field('value', $._value),
      repeat($._value),
    ),

    interpolation: $ => seq('#{', $._value, repeat($._value), '}'),

    placeholder: $ => seq('%', choice($.identifier, $._concatenated_identifier)),

    // `-$x`, `+$x`: a signed variable is one token, so `$a -$b` is a list.
    _signed_variable: _ => token(seq(/[-+]/, /([a-zA-Z_]+\.)?\$[a-zA-Z-_][a-zA-Z0-9-_]*/)),

    // Arguments may be keyword arguments (`$key: value`), rest arguments
    // (`$args...`), and `if(sass(cond): a; else: b)`'s `cond: value` pairs.
    arguments: $ => seq(
      token.immediate('('),
      sep(choice(',', ';'), seq(
        repeat1($._value),
        optional(choice('...', seq(':', repeat1($._value)))),
      )),
      ')',
    ),

    feature_query: $ => seq(
      '(',
      alias(choice($.identifier, $._concatenated_identifier), $.feature_name),
      ':',
      repeat1($._value),
      ')',
    ),

    _concatenated_identifier: $ => choice(
      seq(
        $.identifier,
        repeat1(seq(
          $._concat,
          choice($.interpolation, $.identifier, alias(token.immediate(/-[a-zA-Z0-9_-]*|[0-9][a-zA-Z0-9_-]*/), $.identifier)),
        )),
      ),
      seq(
        $.interpolation,
        repeat(seq(
          $._concat,
          choice($.interpolation, $.identifier, alias(token.immediate(/-[a-zA-Z0-9_-]*|[0-9][a-zA-Z0-9_-]*/), $.identifier)),
        )),
      ),
    ),

    variable: _ => /([a-zA-Z_]+\.)?\$[a-zA-Z-_][a-zA-Z0-9-_]*/,
  },
});

/**
 * Creates a rule to match one or more of the rules separated by `separator`
 *
 * @param {RuleOrLiteral} separator
 *
 * @param {RuleOrLiteral} rule
 *
 * @return {SeqRule}
 *
 */
function sep1(separator, rule) {
  return seq(rule, repeat(seq(separator, rule)));
}

/**
 * Creates a rule to match two or more of the rules separated by `separator`
 *
 * @param {RuleOrLiteral} separator
 *
 * @param {RuleOrLiteral} rules
 *
 * @return {SeqRule}
 */
function sep2(separator, rules) {
  return seq(rules, repeat1(seq(separator, rules)));
}

/**
 * Creates a rule to optionally match one or more of the rules separated by `separator`
 *
 * @param {RuleOrLiteral} separator
 *
 * @param {RuleOrLiteral} rule
 *
 * @return {ChoiceRule}
 */
function sep(separator, rule) {
  return optional(sep1(separator, rule));
}
