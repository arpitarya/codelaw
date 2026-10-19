//! Names a language defines, which no file declares.

/// ECMAScript's global object properties (ECMA-262 §19), plus `arguments`.
/// Environment globals (browser, Node) are not here: a rule names them
/// from its pack's data tables.
pub const ES_GLOBALS: &[&str] = &[
    "AggregateError", "Array", "ArrayBuffer", "AsyncDisposableStack", "Atomics",
    "BigInt", "BigInt64Array", "BigUint64Array", "Boolean", "DataView", "Date", "DisposableStack",
    "Error", "EvalError", "FinalizationRegistry", "Float16Array", "Float32Array", "Float64Array",
    "Function", "Infinity", "Int16Array", "Int32Array", "Int8Array", "Intl", "Iterator", "JSON",
    "Map", "Math", "NaN", "Number", "Object", "Promise", "Proxy", "RangeError", "ReferenceError",
    "Reflect", "RegExp", "Set", "SharedArrayBuffer", "String", "SuppressedError", "Symbol",
    "SyntaxError", "TypeError", "URIError", "Uint16Array", "Uint32Array", "Uint8Array",
    "Uint8ClampedArray", "WeakMap", "WeakRef", "WeakSet", "arguments", "decodeURI",
    "decodeURIComponent", "encodeURI", "encodeURIComponent", "escape", "eval", "globalThis",
    "isFinite", "isNaN", "parseFloat", "parseInt", "undefined", "unescape",
];

/// TypeScript's built-in types (`lib.es5.d.ts` and later), which a type
/// reference may name without importing.
pub const TS_GLOBAL_TYPES: &[&str] = &[
    "ArrayBufferLike", "ArrayBufferView", "ArrayLike", "AsyncDisposable", "AsyncGenerator",
    "AsyncIterable", "AsyncIterableIterator", "Awaited", "CallableFunction", "Capitalize",
    "ClassDecorator", "ConstructorParameters", "Disposable", "Exclude", "Extract", "Generator",
    "IArguments", "InstanceType", "Iterable", "IterableIterator", "Lowercase", "MethodDecorator",
    "NewableFunction", "NoInfer", "NonNullable", "Omit", "OmitThisParameter", "ParameterDecorator",
    "Parameters", "Partial", "Pick", "PromiseLike", "PropertyDecorator", "PropertyDescriptor",
    "PropertyDescriptorMap", "PropertyKey", "Readonly", "ReadonlyArray", "ReadonlyMap",
    "ReadonlySet", "Record", "Required", "ReturnType", "TemplateStringsArray", "ThisParameterType",
    "ThisType", "TypedPropertyDescriptor", "Uncapitalize", "Uppercase", "WeakKey",
];

/// Python's builtins (3.9's `dir(builtins)` plus later additions) and the
/// module-level dunders every module has.
pub const PYTHON_BUILTINS: &[&str] = &[
    "ArithmeticError", "AssertionError", "AttributeError", "BaseException", "BaseExceptionGroup",
    "BlockingIOError", "BrokenPipeError", "BufferError", "BytesWarning", "ChildProcessError",
    "ConnectionAbortedError", "ConnectionError", "ConnectionRefusedError", "ConnectionResetError",
    "DeprecationWarning", "EOFError", "Ellipsis", "EncodingWarning", "EnvironmentError",
    "Exception", "ExceptionGroup", "False", "FileExistsError", "FileNotFoundError",
    "FloatingPointError", "FutureWarning", "GeneratorExit", "IOError", "ImportError",
    "ImportWarning", "IndentationError", "IndexError", "InterruptedError", "IsADirectoryError",
    "KeyError", "KeyboardInterrupt", "LookupError", "MemoryError", "ModuleNotFoundError",
    "NameError", "None", "NotADirectoryError", "NotImplemented", "NotImplementedError", "OSError",
    "OverflowError", "PendingDeprecationWarning", "PermissionError", "ProcessLookupError",
    "PythonFinalizationError", "RecursionError", "ReferenceError", "ResourceWarning",
    "RuntimeError", "RuntimeWarning", "StopAsyncIteration", "StopIteration", "SyntaxError",
    "SyntaxWarning", "SystemError", "SystemExit", "TabError", "TimeoutError", "True", "TypeError",
    "UnboundLocalError", "UnicodeDecodeError", "UnicodeEncodeError", "UnicodeError",
    "UnicodeTranslateError", "UnicodeWarning", "UserWarning", "ValueError", "Warning",
    "ZeroDivisionError", "_IncompleteInputError", "__annotations__", "__build_class__",
    "__builtins__", "__cached__", "__debug__", "__dict__", "__doc__", "__file__", "__import__",
    "__loader__", "__name__", "__package__", "__path__", "__spec__", "abs", "aiter", "all",
    "anext", "any", "ascii", "bin", "bool", "breakpoint", "bytearray", "bytes", "callable", "chr",
    "classmethod", "compile", "complex", "copyright", "credits", "delattr", "dict", "dir",
    "divmod", "enumerate", "eval", "exec", "exit", "filter", "float", "format", "frozenset",
    "getattr", "globals", "hasattr", "hash", "help", "hex", "id", "input", "int", "isinstance",
    "issubclass", "iter", "len", "license", "list", "locals", "map", "max", "memoryview", "min",
    "next", "object", "oct", "open", "ord", "pow", "print", "property", "quit", "range", "repr",
    "reversed", "round", "set", "setattr", "slice", "sorted", "staticmethod", "str", "sum",
    "super", "tuple", "type", "vars", "zip",
];
