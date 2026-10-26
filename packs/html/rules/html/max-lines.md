---
id: html/max-lines
title: "A page stays under a line limit"
languages: [html]
fix: manual
fix_reason: "splitting a page into partials or components changes its structure"
stability: preview
options:
  max: { type: "integer", default: 400, description: "the most lines a page or template may have" }
  skip_blank_lines: { type: "boolean", default: true, description: "blank lines do not count" }
  skip_comments: { type: "boolean", default: true, description: "comment-only lines do not count: `<!-- -->`" }
  skip_embedded: { type: "boolean", default: false, description: "lines inside `<script>` and `<style>` do not count (the JS and CSS rules check their content)" }
ported_from: []
since: 0.2.0
---

An HTML file has at most `max` lines (400 by default). It applies to whole
`.html` and `.htm` files only.

**Why:** A very long page is hard to read and review and usually holds parts
that want their own component, partial or include. Neither html-validate nor
HTMLHint has a rule for it, so these defaults are codelaw's own. The
`html/size` preset turns it on.

A line holding markup and a comment counts as markup. With `skip_embedded`,
only the lines wholly inside a `<script>` or `<style>` body are left out; the
tag lines count. The finding is reported once, on the first line over the
limit: "412 lines (max 400; blank lines and comments not counted)".

```codelaw
native: html/max-lines
```

## How to fix

1. Pull a self-contained part (header, footer, a card, a form) into a partial,
   include or component.
2. Move a large inline `<script>` or `<style>` into its own file and link it.
3. If the limit is wrong for these files, raise it for them with a `[scope]`
   table rather than for the project.

## Examples

### HTML

#### Don't

A page of 401 lines, one over the default limit (the example is long on
purpose):

```html
<p>Line 1</p>
<p>Line 2</p>
<p>Line 3</p>
<p>Line 4</p>
<p>Line 5</p>
<p>Line 6</p>
<p>Line 7</p>
<p>Line 8</p>
<p>Line 9</p>
<p>Line 10</p>
<p>Line 11</p>
<p>Line 12</p>
<p>Line 13</p>
<p>Line 14</p>
<p>Line 15</p>
<p>Line 16</p>
<p>Line 17</p>
<p>Line 18</p>
<p>Line 19</p>
<p>Line 20</p>
<p>Line 21</p>
<p>Line 22</p>
<p>Line 23</p>
<p>Line 24</p>
<p>Line 25</p>
<p>Line 26</p>
<p>Line 27</p>
<p>Line 28</p>
<p>Line 29</p>
<p>Line 30</p>
<p>Line 31</p>
<p>Line 32</p>
<p>Line 33</p>
<p>Line 34</p>
<p>Line 35</p>
<p>Line 36</p>
<p>Line 37</p>
<p>Line 38</p>
<p>Line 39</p>
<p>Line 40</p>
<p>Line 41</p>
<p>Line 42</p>
<p>Line 43</p>
<p>Line 44</p>
<p>Line 45</p>
<p>Line 46</p>
<p>Line 47</p>
<p>Line 48</p>
<p>Line 49</p>
<p>Line 50</p>
<p>Line 51</p>
<p>Line 52</p>
<p>Line 53</p>
<p>Line 54</p>
<p>Line 55</p>
<p>Line 56</p>
<p>Line 57</p>
<p>Line 58</p>
<p>Line 59</p>
<p>Line 60</p>
<p>Line 61</p>
<p>Line 62</p>
<p>Line 63</p>
<p>Line 64</p>
<p>Line 65</p>
<p>Line 66</p>
<p>Line 67</p>
<p>Line 68</p>
<p>Line 69</p>
<p>Line 70</p>
<p>Line 71</p>
<p>Line 72</p>
<p>Line 73</p>
<p>Line 74</p>
<p>Line 75</p>
<p>Line 76</p>
<p>Line 77</p>
<p>Line 78</p>
<p>Line 79</p>
<p>Line 80</p>
<p>Line 81</p>
<p>Line 82</p>
<p>Line 83</p>
<p>Line 84</p>
<p>Line 85</p>
<p>Line 86</p>
<p>Line 87</p>
<p>Line 88</p>
<p>Line 89</p>
<p>Line 90</p>
<p>Line 91</p>
<p>Line 92</p>
<p>Line 93</p>
<p>Line 94</p>
<p>Line 95</p>
<p>Line 96</p>
<p>Line 97</p>
<p>Line 98</p>
<p>Line 99</p>
<p>Line 100</p>
<p>Line 101</p>
<p>Line 102</p>
<p>Line 103</p>
<p>Line 104</p>
<p>Line 105</p>
<p>Line 106</p>
<p>Line 107</p>
<p>Line 108</p>
<p>Line 109</p>
<p>Line 110</p>
<p>Line 111</p>
<p>Line 112</p>
<p>Line 113</p>
<p>Line 114</p>
<p>Line 115</p>
<p>Line 116</p>
<p>Line 117</p>
<p>Line 118</p>
<p>Line 119</p>
<p>Line 120</p>
<p>Line 121</p>
<p>Line 122</p>
<p>Line 123</p>
<p>Line 124</p>
<p>Line 125</p>
<p>Line 126</p>
<p>Line 127</p>
<p>Line 128</p>
<p>Line 129</p>
<p>Line 130</p>
<p>Line 131</p>
<p>Line 132</p>
<p>Line 133</p>
<p>Line 134</p>
<p>Line 135</p>
<p>Line 136</p>
<p>Line 137</p>
<p>Line 138</p>
<p>Line 139</p>
<p>Line 140</p>
<p>Line 141</p>
<p>Line 142</p>
<p>Line 143</p>
<p>Line 144</p>
<p>Line 145</p>
<p>Line 146</p>
<p>Line 147</p>
<p>Line 148</p>
<p>Line 149</p>
<p>Line 150</p>
<p>Line 151</p>
<p>Line 152</p>
<p>Line 153</p>
<p>Line 154</p>
<p>Line 155</p>
<p>Line 156</p>
<p>Line 157</p>
<p>Line 158</p>
<p>Line 159</p>
<p>Line 160</p>
<p>Line 161</p>
<p>Line 162</p>
<p>Line 163</p>
<p>Line 164</p>
<p>Line 165</p>
<p>Line 166</p>
<p>Line 167</p>
<p>Line 168</p>
<p>Line 169</p>
<p>Line 170</p>
<p>Line 171</p>
<p>Line 172</p>
<p>Line 173</p>
<p>Line 174</p>
<p>Line 175</p>
<p>Line 176</p>
<p>Line 177</p>
<p>Line 178</p>
<p>Line 179</p>
<p>Line 180</p>
<p>Line 181</p>
<p>Line 182</p>
<p>Line 183</p>
<p>Line 184</p>
<p>Line 185</p>
<p>Line 186</p>
<p>Line 187</p>
<p>Line 188</p>
<p>Line 189</p>
<p>Line 190</p>
<p>Line 191</p>
<p>Line 192</p>
<p>Line 193</p>
<p>Line 194</p>
<p>Line 195</p>
<p>Line 196</p>
<p>Line 197</p>
<p>Line 198</p>
<p>Line 199</p>
<p>Line 200</p>
<p>Line 201</p>
<p>Line 202</p>
<p>Line 203</p>
<p>Line 204</p>
<p>Line 205</p>
<p>Line 206</p>
<p>Line 207</p>
<p>Line 208</p>
<p>Line 209</p>
<p>Line 210</p>
<p>Line 211</p>
<p>Line 212</p>
<p>Line 213</p>
<p>Line 214</p>
<p>Line 215</p>
<p>Line 216</p>
<p>Line 217</p>
<p>Line 218</p>
<p>Line 219</p>
<p>Line 220</p>
<p>Line 221</p>
<p>Line 222</p>
<p>Line 223</p>
<p>Line 224</p>
<p>Line 225</p>
<p>Line 226</p>
<p>Line 227</p>
<p>Line 228</p>
<p>Line 229</p>
<p>Line 230</p>
<p>Line 231</p>
<p>Line 232</p>
<p>Line 233</p>
<p>Line 234</p>
<p>Line 235</p>
<p>Line 236</p>
<p>Line 237</p>
<p>Line 238</p>
<p>Line 239</p>
<p>Line 240</p>
<p>Line 241</p>
<p>Line 242</p>
<p>Line 243</p>
<p>Line 244</p>
<p>Line 245</p>
<p>Line 246</p>
<p>Line 247</p>
<p>Line 248</p>
<p>Line 249</p>
<p>Line 250</p>
<p>Line 251</p>
<p>Line 252</p>
<p>Line 253</p>
<p>Line 254</p>
<p>Line 255</p>
<p>Line 256</p>
<p>Line 257</p>
<p>Line 258</p>
<p>Line 259</p>
<p>Line 260</p>
<p>Line 261</p>
<p>Line 262</p>
<p>Line 263</p>
<p>Line 264</p>
<p>Line 265</p>
<p>Line 266</p>
<p>Line 267</p>
<p>Line 268</p>
<p>Line 269</p>
<p>Line 270</p>
<p>Line 271</p>
<p>Line 272</p>
<p>Line 273</p>
<p>Line 274</p>
<p>Line 275</p>
<p>Line 276</p>
<p>Line 277</p>
<p>Line 278</p>
<p>Line 279</p>
<p>Line 280</p>
<p>Line 281</p>
<p>Line 282</p>
<p>Line 283</p>
<p>Line 284</p>
<p>Line 285</p>
<p>Line 286</p>
<p>Line 287</p>
<p>Line 288</p>
<p>Line 289</p>
<p>Line 290</p>
<p>Line 291</p>
<p>Line 292</p>
<p>Line 293</p>
<p>Line 294</p>
<p>Line 295</p>
<p>Line 296</p>
<p>Line 297</p>
<p>Line 298</p>
<p>Line 299</p>
<p>Line 300</p>
<p>Line 301</p>
<p>Line 302</p>
<p>Line 303</p>
<p>Line 304</p>
<p>Line 305</p>
<p>Line 306</p>
<p>Line 307</p>
<p>Line 308</p>
<p>Line 309</p>
<p>Line 310</p>
<p>Line 311</p>
<p>Line 312</p>
<p>Line 313</p>
<p>Line 314</p>
<p>Line 315</p>
<p>Line 316</p>
<p>Line 317</p>
<p>Line 318</p>
<p>Line 319</p>
<p>Line 320</p>
<p>Line 321</p>
<p>Line 322</p>
<p>Line 323</p>
<p>Line 324</p>
<p>Line 325</p>
<p>Line 326</p>
<p>Line 327</p>
<p>Line 328</p>
<p>Line 329</p>
<p>Line 330</p>
<p>Line 331</p>
<p>Line 332</p>
<p>Line 333</p>
<p>Line 334</p>
<p>Line 335</p>
<p>Line 336</p>
<p>Line 337</p>
<p>Line 338</p>
<p>Line 339</p>
<p>Line 340</p>
<p>Line 341</p>
<p>Line 342</p>
<p>Line 343</p>
<p>Line 344</p>
<p>Line 345</p>
<p>Line 346</p>
<p>Line 347</p>
<p>Line 348</p>
<p>Line 349</p>
<p>Line 350</p>
<p>Line 351</p>
<p>Line 352</p>
<p>Line 353</p>
<p>Line 354</p>
<p>Line 355</p>
<p>Line 356</p>
<p>Line 357</p>
<p>Line 358</p>
<p>Line 359</p>
<p>Line 360</p>
<p>Line 361</p>
<p>Line 362</p>
<p>Line 363</p>
<p>Line 364</p>
<p>Line 365</p>
<p>Line 366</p>
<p>Line 367</p>
<p>Line 368</p>
<p>Line 369</p>
<p>Line 370</p>
<p>Line 371</p>
<p>Line 372</p>
<p>Line 373</p>
<p>Line 374</p>
<p>Line 375</p>
<p>Line 376</p>
<p>Line 377</p>
<p>Line 378</p>
<p>Line 379</p>
<p>Line 380</p>
<p>Line 381</p>
<p>Line 382</p>
<p>Line 383</p>
<p>Line 384</p>
<p>Line 385</p>
<p>Line 386</p>
<p>Line 387</p>
<p>Line 388</p>
<p>Line 389</p>
<p>Line 390</p>
<p>Line 391</p>
<p>Line 392</p>
<p>Line 393</p>
<p>Line 394</p>
<p>Line 395</p>
<p>Line 396</p>
<p>Line 397</p>
<p>Line 398</p>
<p>Line 399</p>
<p>Line 400</p>
<p>Line 401</p>
```

#### Do

Split it, so each file stays short:

```html
<main>
  <p>Line 1</p>
  <p>Line 2</p>
</main>
```
