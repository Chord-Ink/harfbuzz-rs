# Clusters

*Adapted from the HarfBuzz 14.3.0 documentation, which is licensed under the terms in the HarfBuzz COPYING file.*

## Clusters and shaping

In text shaping, a *cluster* is a sequence of characters that needs to be treated as a single, indivisible unit. A single letter or symbol can be a cluster of its own. Other clusters correspond to longer subsequences of the input code points — such as a ligature or conjunct form — and require the shaper to ensure that the cluster is not broken during the shaping process.

A cluster is distinct from a *grapheme*, which is the smallest unit of meaning in a writing system or script.

The definitions of the two terms are similar. However, clusters are only relevant for script shaping and glyph layout. In contrast, graphemes are a property of the underlying script, and are of interest when client programs implement orthographic or linguistic functionality.

For example, two individual letters are often two separate graphemes. When two letters form a ligature, however, they combine into a single glyph. They are then part of the same cluster and are treated as a unit by the shaping engine — even though the two original, underlying letters remain separate graphemes.

HarfBuzz is concerned with clusters, *not* with graphemes — although client programs using HarfBuzz may still care about graphemes for other reasons from time to time.

During the shaping process, there are several shaping operations that may merge adjacent characters (for example, when two code points form a ligature or a conjunct form and are replaced by a single glyph) or split one character into several (for example, when decomposing a code point through the `ccmp` feature). Operations like these alter clusters; HarfBuzz tracks the changes to ensure that no clusters get lost or broken during shaping.

HarfBuzz records cluster information independently from how shaping operations affect the individual glyphs returned in an output buffer. Consequently, a client program using HarfBuzz can utilize the cluster information to implement features such as:

- Correctly positioning the cursor within a shaped text run, even when characters have formed ligatures, composed or decomposed, reordered, or undergone other shaping operations.

- Correctly highlighting a text selection that includes some, but not all, of the characters in a word.

- Applying text attributes (such as color or underlining) to part, but not all, of a word.

- Generating output document formats (such as PDF) with embedded text that can be fully extracted.

- Determining the mapping between input characters and output glyphs, such as which glyphs are ligatures.

- Performing line-breaking, justification, and other line-level or paragraph-level operations that must be done after shaping is complete, but which require examining character-level properties.

## Working with HarfBuzz clusters

When you add text to a HarfBuzz buffer, each code point must be assigned a *cluster value*.

This cluster value is an arbitrary number; HarfBuzz uses it only to distinguish between clusters. Many client programs will use the index of each code point in the input text stream as the cluster value. This is for the sake of convenience; the actual value does not matter.

Some of the shaping operations performed by HarfBuzz — such as reordering, composition, decomposition, and substitution — may alter the cluster values of some characters. The final cluster values in the buffer at the end of the shaping process will indicate to client programs which subsequences of glyphs represent a cluster and, therefore, must not be separated.

In addition, client programs can query the final cluster values to discern other potentially important information about the glyphs in the output buffer (such as whether or not a ligature was formed).

For example, if the initial sequence of cluster values was:

```text
0,1,2,3,4
```

and the final sequence of cluster values is:

```text
0,0,3,3
```

then there are two clusters in the output buffer: the first cluster includes the first two glyphs, and the second cluster includes the third and fourth glyphs. It is also evident that a ligature or conjunct has been formed, because there are fewer glyphs in the output buffer (four) than there were code points in the input buffer (five).

Although client programs using HarfBuzz are free to assign initial cluster values in any manner they choose to, HarfBuzz does offer some useful guarantees if the cluster values are assigned in a monotonic (either non-decreasing or non-increasing) order.

For buffers in the left-to-right (LTR) or top-to-bottom (TTB) text flow direction, HarfBuzz will preserve the monotonic property: client programs are guaranteed that monotonically increasing initial cluster values will be returned as monotonically increasing final cluster values.

For buffers in the right-to-left (RTL) or bottom-to-top (BTT) text flow direction, the directionality of the buffer itself is reversed for final output as a matter of design. Therefore, HarfBuzz inverts the monotonic property: client programs are guaranteed that monotonically increasing initial cluster values will be returned as monotonically *decreasing* final cluster values.

Client programs can adjust how HarfBuzz handles clusters during shaping by setting the `cluster_level` of the buffer. HarfBuzz offers three *levels* of clustering support for this property:

- *Level 0* is the default.

  The distinguishing feature of level 0 behavior is that, at the beginning of processing the buffer, all code points that are categorized as *marks*, *modifier symbols*, or *Emoji extended pictographic* modifiers, as well as the *Zero Width Joiner* and *Zero Width Non-Joiner* code points, are assigned the cluster value of the closest preceding code point from *different* category.

  In essence, whenever a base character is followed by a mark character or a sequence of mark characters, those marks are reassigned to the same initial cluster value as the base character. This reassignment is referred to as "merging" the affected clusters. This behavior is based on the Grapheme Cluster Boundary specification in [Unicode Technical Report 29](https://www.unicode.org/reports/tr29/#Regex_Definitions).

  This cluster level is suitable for code that likes to use HarfBuzz cluster values as an approximation of the Unicode Grapheme Cluster Boundaries as well.

  Client programs can specify level 0 behavior for a buffer by setting its `cluster_level` to `HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES`.

- *Level 1* tweaks the old behavior slightly to produce better results. Therefore, level 1 clustering is recommended for code that is not required to implement backward compatibility with the old HarfBuzz.

  *Level 1* differs from level 0 by not merging the clusters of marks and other modifier code points with the preceding "base" code point's cluster. By preserving the separate cluster values of these marks and modifier code points, script shapers can perform additional operations that might lead to improved results (for example, coloring mark glyphs differently than their base).

  Client programs can specify level 1 behavior for a buffer by setting its `cluster_level` to `HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS`.

- *Level 2* differs significantly in how it treats cluster values. In level 2, HarfBuzz never merges clusters.

  This difference can be seen most clearly when HarfBuzz processes ligature substitutions and glyph decompositions. In level 0 and level 1, ligatures and glyph decomposition both involve merging clusters; in level 2, neither of these operations triggers a merge.

  Client programs can specify level 2 behavior for a buffer by setting its `cluster_level` to `HB_BUFFER_CLUSTER_LEVEL_CHARACTERS`.

As mentioned earlier, client programs using HarfBuzz often assign initial cluster values in a buffer by reusing the indices of the code points in the input text. This gives a sequence of cluster values that is monotonically increasing (for example, 0,1,2,3,4).

It is not *required* that the cluster values in a buffer be monotonically increasing. However, if the initial cluster values in a buffer are monotonic and the buffer is configured to use cluster level 0 or 1, then HarfBuzz guarantees that the final cluster values in the shaped buffer will also be monotonic. No such guarantee is made for cluster level 2.

In levels 0 and 1, HarfBuzz implements the following conceptual model for cluster values:

- If the sequence of input cluster values is monotonic, the sequence of cluster values will remain monotonic.
- Each cluster value represents a single cluster.
- Each cluster contains one or more glyphs and one or more characters.

In practice, this model offers several benefits. Assuming that the initial cluster values were monotonically increasing and distinct before shaping began, then, in the final output:

- All adjacent glyphs having the same final cluster value belong to the same cluster.
- Each character belongs to the cluster that has the highest cluster value *not larger than* its initial cluster value.

## A clustering example for levels 0 and 1

The basic shaping operations affect clusters in a predictable manner when using level 0 or level 1:

- When two or more clusters *merge*, the resulting merged cluster takes as its cluster value the *minimum* of the incoming cluster values.

- When a cluster *decomposes*, all of the resulting child clusters inherit as their cluster value the cluster value of the parent cluster.

- When a character is *reordered*, the reordered character and all clusters that the character moves past as part of the reordering are merged into one cluster.

The functionality, guarantees, and benefits of level 0 and level 1 behavior can be seen with some examples. First, let us examine what happens with cluster values when shaping involves cluster merging with ligatures and decomposition.

Let's say we start with the following character sequence (top row) and initial cluster values (bottom row):

```text
A,B,C,D,E
0,1,2,3,4
```

During shaping, HarfBuzz maps these characters to glyphs from the font. For simplicity, let us assume that each character maps to the corresponding, identical-looking glyph:

```text
A,B,C,D,E
0,1,2,3,4
```

Now if, for example, `B` and `C` form a ligature, then the clusters to which they belong "merge". This merged cluster takes for its cluster value the minimum of all the cluster values of the clusters that went in to the ligature. In this case, we get:

```text
A,BC,D,E
0,1 ,3,4
```

because 1 is the minimum of the set {1,2}, which were the cluster values of `B` and `C`.

Next, let us say that the `BC` ligature glyph decomposes into three components, and `D` also decomposes into two components. Whenever a cluster decomposes, its components each inherit the cluster value of their parent:

```text
A,BC0,BC1,BC2,D0,D1,E
0,1  ,1  ,1  ,3 ,3 ,4
```

Next, if `BC2` and `D0` form a ligature, then their clusters (cluster values 1 and 3) merge into `min(1,3) = 1`:

```text
A,BC0,BC1,BC2D0,D1,E
0,1  ,1  ,1    ,1 ,4
```

Note that the entirety of cluster 3 merges into cluster 1, not just the `D0` glyph. This reflects the fact that the cluster *must* be treated as an indivisible unit.

At this point, cluster 1 means: the character sequence `BCD` is represented by glyphs `BC0,BC1,BC2D0,D1` and cannot be broken down any further.

## Reordering in levels 0 and 1

Another common operation in some shapers is glyph reordering. In order to maintain a monotonic cluster sequence when glyph reordering takes place, HarfBuzz merges the clusters of everything in the reordering sequence.

For example, let us again start with the character sequence (top row) and initial cluster values (bottom row):

```text
A,B,C,D,E
0,1,2,3,4
```

If `D` is reordered to the position immediately before `B`, then HarfBuzz merges the `B`, `C`, and `D` clusters — all the clusters between the final position of the reordered glyph and its original position. This means that we get:

```text
A,D,B,C,E
0,1,1,1,4
```

as the final cluster sequence.

Merging this many clusters is not ideal, but it is the only sensible way for HarfBuzz to maintain the guarantee that the sequence of cluster values remains monotonic and to retain the true relationship between glyphs and characters.

## The distinction between levels 0 and 1

The preceding examples demonstrate the main effects of using cluster levels 0 and 1. The only difference between the two levels is this: in level 0, at the very beginning of the shaping process, HarfBuzz merges the cluster of each base character with the clusters of all Unicode marks (combining or not) and modifiers that follow it.

For example, let us start with the following character sequence (top row) and accompanying initial cluster values (bottom row):

```text
A,acute,B
0,1    ,2
```

The `acute` is a Unicode mark. If HarfBuzz is using cluster level 0 on this sequence, then the `A` and `acute` clusters will merge, and the result will become:

```text
A,acute,B
0,0    ,2
```

This merger is performed before any other script-shaping steps.

This initial cluster merging is the default behavior of the Windows shaping engine, and the old HarfBuzz codebase copied that behavior to maintain compatibility. Consequently, it has remained the default behavior in the new HarfBuzz codebase.

But this initial cluster-merging behavior makes it impossible for client programs to implement some features (such as to color diacritic marks differently from their base characters). That is why, in level 1, HarfBuzz does not perform the initial merging step.

For client programs that rely on HarfBuzz cluster values to perform cursor positioning, level 0 is more convenient. But relying on cluster boundaries for cursor positioning is wrong: cursor positions should be determined based on Unicode grapheme boundaries, not on shaping-cluster boundaries. As such, using level 1 clustering behavior is recommended.

One final facet of levels 0 and 1 is worth noting. HarfBuzz currently does not allow any *multiple-substitution* GSUB lookups to replace a glyph with zero glyphs (in other words, to delete a glyph).

But, in some other situations, glyphs can be deleted. In those cases, if the glyph being deleted is the last glyph of its cluster, HarfBuzz makes sure to merge the deleted glyph's cluster with a neighboring cluster.

This is done primarily to make sure that the starting cluster of the text always has the cluster index pointing to the start of the text for the run; more than one client program currently relies on this guarantee.

Incidentally, Apple's CoreText does something different to maintain the same promise: it inserts a glyph with id 65535 at the beginning of the glyph string if the glyph corresponding to the first character in the run was deleted. HarfBuzz might do something similar in the future.

## Level 2

HarfBuzz's level 2 cluster behavior uses a significantly different model than that of level 0 and level 1.

The level 2 behavior is easy to describe, but it may be difficult to understand in practical terms. In brief, level 2 performs no merging of clusters whatsoever.

This means that there is no initial base-and-mark merging step (as is done in level 0), and it means that reordering moves and ligature substitutions do not trigger a cluster merge.

Only one shaping operation directly affects clusters when using level 2:

- When a cluster *decomposes*, all of the resulting child clusters inherit as their cluster value the cluster value of the parent cluster.

When glyphs do form a ligature (or when some other feature substitutes multiple glyphs with one glyph) the cluster value of the first glyph is retained as the cluster value for the resulting ligature.

This occurrence sounds similar to a cluster merge, but it is different. In particular, no subsequent characters — including marks and modifiers — are affected. They retain their previous cluster values.

Level 2 cluster behavior is ultimately less complex than level 0 or level 1, but there are several cases for which processing cluster values produced at level 2 may be tricky.

### Ligatures with combining marks in level 2

The first example of how HarfBuzz's level 2 cluster behavior can be tricky is when the text to be shaped includes combining marks attached to ligatures.

Let us start with an input sequence with the following characters (top row) and initial cluster values (bottom row):

```text
A,acute,B,breve,C,circumflex
0,1    ,2,3    ,4,5
```

If the sequence `A,B,C` forms a ligature, then these are the cluster values HarfBuzz will return under the various cluster levels:

Level 0:

```text
ABC,acute,breve,circumflex
0  ,0    ,0    ,0
```

Level 1:

```text
ABC,acute,breve,circumflex
0  ,0    ,0    ,5
```

Level 2:

```text
ABC,acute,breve,circumflex
0  ,1    ,3    ,5
```

Making sense of the level 2 result is the hardest for a client program, because there is nothing in the cluster values that indicates that `B` and `C` formed a ligature with `A`.

In contrast, the "merged" cluster values of the mark glyphs that are seen in the level 0 and level 1 output are evidence that a ligature substitution took place.

### Reordering in level 2

Another example of how HarfBuzz's level 2 cluster behavior can be tricky is when glyphs reorder. Consider an input sequence with the following characters (top row) and initial cluster values (bottom row):

```text
A,B,C,D,E
0,1,2,3,4
```

Now imagine `D` moves before `B` in a reordering operation. The cluster values will then be:

```text
A,D,B,C,E
0,3,1,2,4
```

Next, if `D` forms a ligature with `B`, the output is:

```text
A,DB,C,E
0,3 ,2,4
```

However, in a different scenario, in which the shaping rules of the script instead caused `A` and `B` to form a ligature *before* the `D` reordered, the result would be:

```text
AB,D,C,E
0 ,3,2,4
```

There is no way for a client program to differentiate between these two scenarios based on the cluster values alone. Consequently, client programs that use level 2 might need to undertake additional work in order to manage cursor positioning, text attributes, or other desired features.

### Other considerations in level 2

There may be other problems encountered with ligatures under level 2, such as if the direction of the text is forced to the opposite of its natural direction (for example, Arabic text that is forced into left-to-right directionality). But, generally speaking, these other scenarios are minor corner cases that are too obscure for most client programs to need to worry about.
