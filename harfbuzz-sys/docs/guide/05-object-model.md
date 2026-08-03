# The HarfBuzz object model

*Adapted from the HarfBuzz 14.3.0 documentation, which is licensed under the terms in the HarfBuzz `COPYING` file.*

## An overview of data types in HarfBuzz

HarfBuzz features two kinds of data types: non-opaque, pass-by-value types and opaque, heap-allocated types. This kind of separation is common in C libraries that have to provide API/ABI compatibility (almost) indefinitely.

*Value types:* The non-opaque, pass-by-value types include integer types, enums, and small structs. Exposing a struct in the public API makes it impossible to expand the struct in the future. As such, exposing structs is reserved for cases where it's extremely inefficient to do otherwise.

In HarfBuzz, several structs, like `hb_glyph_info_t` and `hb_glyph_position_t`, fall into that efficiency-sensitive category and are non-opaque.

For all non-opaque structs where future extensibility may be necessary, reserved members are included to hold space for possible future members. As such, it's important to provide `equal()`, and `hash()` methods for such structs, allowing users of the API do effectively deal with the type without having to adapt their code to future changes.

Important value types provided by HarfBuzz include the structs for working with Unicode code points, glyphs, and tags for font tables and features, as well as the enums for many Unicode and OpenType properties.

## Objects in HarfBuzz

*Object types:* Opaque struct types are used for what HarfBuzz loosely calls "objects." This doesn't have much to do with the terminology from object-oriented programming (OOP), although some of the concepts are similar.

In HarfBuzz, all object types provide certain lifecycle-management APIs. Objects are reference-counted, and constructed with various `create()` methods, referenced via `reference()` and dereferenced using `destroy()`.

For example, the `hb_buffer_t` object has `hb_buffer_create()` as its constructor, `hb_buffer_reference()` to reference, and `hb_buffer_destroy()` to dereference.

After construction, each object's properties are accessible only through the setter and getter functions described in the API Reference manual.

Note that many object types can be marked as read-only or immutable, facilitating their use in multi-threaded environments.

Key object types provided by HarfBuzz include:

- *blobs*, which act as low-level wrappers around binary data. Blobs are typically used to hold the contents of a binary font file.
- *faces*, which represent typefaces from a font file, but without specific parameters (such as size) set.
- *fonts*, which represent instances of a face with all of their parameters specified.
- *buffers*, which hold Unicode code points for characters (before shaping) and the shaped glyph output (after shaping).
- *shape plans*, which store the settings that HarfBuzz will use when shaping a particular text segment. Shape plans are not generally used by client programs directly, but as we will see in a later guide, they are still valuable to understand.

## Object lifecycle management

Each object type in HarfBuzz provides a `create()` method. Some object types provide additional variants of `create()` to handle special cases or to speed up common tasks; those variants are documented in the API reference. For example, `hb_blob_create_from_file()` constructs a new blob directly from the contents of a file.

All objects are created with an initial reference count of `1`. Client programs can increase the reference count on an object by calling its `reference()` method. Whenever a client program is finished with an object, it should call its corresponding `destroy()` method. The destroy method will decrease the reference count on the object and, whenever the reference count reaches zero, it will also destroy the object and free all of the associated memory.

All of HarfBuzz's object-lifecycle-management APIs are thread-safe (unless you compiled HarfBuzz from source with the `HB_NO_MT` configuration flag), even when the object as a whole is not thread-safe. It is also permissible to `reference()` or to `destroy()` the `NULL` value.

Some objects are thread-safe after they have been constructed and set up. The general pattern is to `create()` the object, make a few `set_*()` calls to set up the object, and then use it without further modification.

To ensure that such an object is not modified, client programs can explicitly mark an object as immutable. HarfBuzz provides `make_immutable()` methods to mark an object as immutable and `is_immutable()` methods to test whether or not an object is immutable. Attempts to use setter functions on immutable objects will fail silently; see the API Reference manual for specifics.

Note also that there are no "make mutable" methods. If client programs need to alter an object previously marked as immutable, they will need to make a duplicate of the original.

Finally, object constructors (and, indeed, as much of the shaping API as possible) will never return `NULL`. Instead, if there is an allocation error, each constructor will return an "empty" object singleton.

These empty-object singletons are inert and safe (although typically useless) to pass around. This design choice avoids having to check for `NULL` pointers all throughout the code.

In addition, this "empty" object singleton can also be accessed using the `get_empty()` method of the object type in question.

## User data

To better integrate with client programs, HarfBuzz's objects offer a "user data" mechanism that can be used to attach arbitrary data to the object. User-data attachment can be useful for tying the lifecycles of various pieces of data together, or for creating language bindings.

Each object type has a `set_user_data()` method and a `get_user_data()` method. The `set_user_data()` methods take a client-provided `key` and a pointer, `user_data`, pointing to the data itself. Once the key-data pair has been attached to the object, the `get_user_data()` method can be called with the key, returning the `user_data` pointer.

The `set_user_data()` methods also support an optional `destroy` callback. Client programs can set the `destroy` callback and receive notification from HarfBuzz whenever the object is destructed.

Finally, each `set_user_data()` method allows the client program to set a `replace` Boolean indicating whether or not the function call should replace any existing `user_data` associated with the specified key.

## Blobs

While most of HarfBuzz's object types are specific to the shaping process, *blobs* are somewhat different.

Blobs are an abstraction designed to negotiate lifecycle and permissions for raw pieces of data. For example, when you load the raw font data into memory and want to pass it to HarfBuzz, you do so in a `hb_blob_t` wrapper.

This allows you to take advantage of HarfBuzz's reference-counting and `destroy` callbacks. If you allocated the memory for the data using `malloc()`, you would create the blob using

```c
hb_blob_create (data, length, HB_MEMORY_MODE_WRITABLE, data, free)
```

That way, HarfBuzz will call `free()` on the allocated memory whenever the blob drops its last reference and is deconstructed. Consequently, the user code can stop worrying about freeing memory and let the reference-counting machinery take care of that.

Most of the time, blobs are read-only, facilitating their use in immutable objects.
