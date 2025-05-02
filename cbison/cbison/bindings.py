# -*- coding: utf-8 -*-
#
# TARGET arch is: ['-isysroot', '/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk']
# WORD_SIZE is: 8
# POINTER_SIZE is: 8
# LONGDOUBLE_SIZE is: 8
#
import ctypes


class AsDictMixin:
    @classmethod
    def as_dict(cls, self):
        result = {}
        if not isinstance(self, AsDictMixin):
            # not a structure, assume it's already a python object
            return self
        if not hasattr(cls, "_fields_"):
            return result
        # sys.version_info >= (3, 5)
        # for (field, *_) in cls._fields_:  # noqa
        for field_tuple in cls._fields_:  # noqa
            field = field_tuple[0]
            if field.startswith('PADDING_'):
                continue
            value = getattr(self, field)
            type_ = type(value)
            if hasattr(value, "_length_") and hasattr(value, "_type_"):
                # array
                type_ = type_._type_
                if hasattr(type_, 'as_dict'):
                    value = [type_.as_dict(v) for v in value]
                else:
                    value = [i for i in value]
            elif hasattr(value, "contents") and hasattr(value, "_type_"):
                # pointer
                try:
                    if not hasattr(type_, "as_dict"):
                        value = value.contents
                    else:
                        type_ = type_._type_
                        value = type_.as_dict(value.contents)
                except ValueError:
                    # nullptr
                    value = None
            elif isinstance(value, AsDictMixin):
                # other structure
                value = type_.as_dict(value)
            result[field] = value
        return result


class Structure(ctypes.Structure, AsDictMixin):

    def __init__(self, *args, **kwds):
        # We don't want to use positional arguments fill PADDING_* fields

        args = dict(zip(self.__class__._field_names_(), args))
        args.update(kwds)
        super(Structure, self).__init__(**args)

    @classmethod
    def _field_names_(cls):
        if hasattr(cls, '_fields_'):
            return (f[0] for f in cls._fields_ if not f[0].startswith('PADDING'))
        else:
            return ()

    @classmethod
    def get_type(cls, field):
        for f in cls._fields_:
            if f[0] == field:
                return f[1]
        return None

    @classmethod
    def bind(cls, bound_fields):
        fields = {}
        for name, type_ in cls._fields_:
            if hasattr(type_, "restype"):
                if name in bound_fields:
                    if bound_fields[name] is None:
                        fields[name] = type_()
                    else:
                        # use a closure to capture the callback from the loop scope
                        fields[name] = (
                            type_((lambda callback: lambda *args: callback(*args))(
                                bound_fields[name]))
                        )
                    del bound_fields[name]
                else:
                    # default callback implementation (does nothing)
                    try:
                        default_ = type_(0).restype().value
                    except TypeError:
                        default_ = None
                    fields[name] = type_((
                        lambda default_: lambda *args: default_)(default_))
            else:
                # not a callback function, use default initialization
                if name in bound_fields:
                    fields[name] = bound_fields[name]
                    del bound_fields[name]
                else:
                    fields[name] = type_()
        if len(bound_fields) != 0:
            raise ValueError(
                "Cannot bind the following unknown callback(s) {}.{}".format(
                    cls.__name__, bound_fields.keys()
            ))
        return cls(**fields)


class Union(ctypes.Union, AsDictMixin):
    pass



c_int128 = ctypes.c_ubyte*16
c_uint128 = c_int128
void = None
if ctypes.sizeof(ctypes.c_longdouble) == 8:
    c_long_double_t = ctypes.c_longdouble
else:
    c_long_double_t = ctypes.c_ubyte*8

def string_cast(char_pointer, encoding='utf-8', errors='strict'):
    value = ctypes.cast(char_pointer, ctypes.c_char_p).value
    if value is not None and encoding is not None:
        value = value.decode(encoding, errors=errors)
    return value


def char_pointer_cast(string, encoding='utf-8'):
    if encoding is not None:
        try:
            string = string.encode(encoding)
        except AttributeError:
            # In Python3, bytes has no encode attribute
            pass
    string = ctypes.c_char_p(string)
    return ctypes.cast(string, ctypes.c_char_p)





class struct_cbison_matcher(Structure):
    pass

cbison_matcher_t = ctypes.POINTER(struct_cbison_matcher)
class struct_cbison_factory(Structure):
    pass

cbison_factory_t = ctypes.POINTER(struct_cbison_factory)
class struct_cbison_tokenizer(Structure):
    pass

cbison_tokenizer_t = ctypes.POINTER(struct_cbison_tokenizer)
cbison_matcher_ptr_t = ctypes.POINTER(struct_cbison_matcher)
cbison_tokenizer_ptr_t = ctypes.POINTER(struct_cbison_tokenizer)
class struct_cbison_mask_req(Structure):
    pass

struct_cbison_mask_req._pack_ = 1 # source:False
struct_cbison_mask_req._fields_ = [
    ('matcher', cbison_matcher_t),
    ('mask_dest', ctypes.POINTER(ctypes.c_uint32)),
]

cbison_mask_req_t = struct_cbison_mask_req
struct_cbison_factory._pack_ = 1 # source:False
struct_cbison_factory._fields_ = [
    ('magic', ctypes.c_uint32),
    ('impl_magic', ctypes.c_uint32),
    ('version_major', ctypes.c_uint32),
    ('version_minor', ctypes.c_uint32),
    ('n_vocab', ctypes.c_size_t),
    ('mask_byte_len', ctypes.c_size_t),
    ('eos_token_id', ctypes.c_uint32),
    ('reserved_hd', ctypes.c_uint32 * 7),
    ('free_factory', ctypes.CFUNCTYPE(None, cbison_factory_t)),
    ('validate_grammar', ctypes.CFUNCTYPE(ctypes.c_int32, cbison_factory_t, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_size_t)),
    ('new_matcher', ctypes.CFUNCTYPE(cbison_matcher_t, cbison_factory_t, ctypes.c_char_p, ctypes.c_char_p)),
    ('get_error', ctypes.CFUNCTYPE(ctypes.c_char_p, cbison_matcher_t)),
    ('compute_mask', ctypes.CFUNCTYPE(ctypes.c_int32, cbison_matcher_t, ctypes.POINTER(ctypes.c_uint32), ctypes.c_size_t)),
    ('consume_tokens', ctypes.CFUNCTYPE(ctypes.c_int32, cbison_matcher_t, ctypes.POINTER(ctypes.c_uint32), ctypes.c_size_t)),
    ('is_accepting', ctypes.CFUNCTYPE(ctypes.c_bool, cbison_matcher_t)),
    ('is_stopped', ctypes.CFUNCTYPE(ctypes.c_bool, cbison_matcher_t)),
    ('validate_tokens', ctypes.CFUNCTYPE(ctypes.c_int32, cbison_matcher_t, ctypes.POINTER(ctypes.c_uint32), ctypes.c_size_t)),
    ('compute_ff_tokens', ctypes.CFUNCTYPE(ctypes.c_int32, cbison_matcher_t, ctypes.POINTER(ctypes.c_uint32), ctypes.c_size_t)),
    ('free_matcher', ctypes.CFUNCTYPE(None, cbison_matcher_t)),
    ('rollback', ctypes.CFUNCTYPE(ctypes.c_int32, cbison_matcher_t, ctypes.c_size_t)),
    ('reset', ctypes.CFUNCTYPE(ctypes.c_int32, cbison_matcher_t)),
    ('clone_matcher', ctypes.CFUNCTYPE(cbison_matcher_t, cbison_matcher_t)),
    ('compute_masks', ctypes.CFUNCTYPE(ctypes.c_int32, cbison_factory_t, ctypes.POINTER(struct_cbison_mask_req), ctypes.c_size_t)),
    ('reserved_ptr', ctypes.POINTER(None) * 16),
]

struct_cbison_tokenizer._pack_ = 1 # source:False
struct_cbison_tokenizer._fields_ = [
    ('magic', ctypes.c_uint32),
    ('impl_magic', ctypes.c_uint32),
    ('version_major', ctypes.c_uint32),
    ('version_minor', ctypes.c_uint32),
    ('n_vocab', ctypes.c_size_t),
    ('eos_token_id', ctypes.c_uint32),
    ('tokenize_bytes_requires_utf8', ctypes.c_bool),
    ('PADDING_0', ctypes.c_ubyte * 3),
    ('reserved_hd', ctypes.c_uint32 * 6),
    ('get_token', ctypes.CFUNCTYPE(ctypes.c_int32, ctypes.POINTER(struct_cbison_tokenizer), ctypes.c_uint32, ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t)),
    ('is_special_token', ctypes.CFUNCTYPE(ctypes.c_int32, ctypes.POINTER(struct_cbison_tokenizer), ctypes.c_uint32)),
    ('tokenize_bytes', ctypes.CFUNCTYPE(ctypes.c_size_t, ctypes.POINTER(struct_cbison_tokenizer), ctypes.POINTER(ctypes.c_ubyte), ctypes.c_uint64, ctypes.POINTER(ctypes.c_uint32), ctypes.c_uint64)),
    ('free_tokenizer', ctypes.CFUNCTYPE(None, ctypes.POINTER(struct_cbison_tokenizer))),
    ('reserved_ptr', ctypes.POINTER(None) * 16),
]

__all__ = \
    ['cbison_factory_t', 'cbison_mask_req_t', 'cbison_matcher_ptr_t',
    'cbison_matcher_t', 'cbison_tokenizer_ptr_t',
    'cbison_tokenizer_t', 'struct_cbison_factory',
    'struct_cbison_mask_req', 'struct_cbison_matcher',
    'struct_cbison_tokenizer']
