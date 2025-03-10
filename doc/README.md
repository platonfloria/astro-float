# Errors

This document describes how error is estimated. Given an operation and arguments containing certain error the goal it to find error bound of the result of the operation. When the error of the result is known, operations can be applied sequentially to estimate error of a more complex construct.

Arguments are represented as $m2^{e_1}$, $n2^{e_2}$, where $m$, $n$ denote the mantissa of a floating point number with $0.5 <= |n| < 1, 0.5 <= |m| < 1$, and $e_1$, $e_2$ denote exponent of a floating point number.

$p$ denotes precision of a floating point number.

$m 2^{e_1} \pm 2^{e_1 - p}$ denotes an argument with absolute error of at most $2^{e_1-p}$, i.e. $\pm1$ ulp.

## Error of multiplication

Absolute error $err_a$ of multiplying numbers containing relative error less than $2^{-p}$:

$$\displaylines{err_a < |(m 2^{e_1} \pm 2^{e_1 - p}) (n 2^{e_2} \pm 2^{e_2 - p}) - m 2^{e_1} n 2^{e_2}| = \\\\ = |(m \pm n) 2^{e_1 + e_2 - p} \pm 2^{e_1 + e_2 - 2 p}| < 2 ^ {e_1 + e_2 - p + 2}}$$

Relative errror:

$$\displaylines{err_r < |\frac{err_a}{mn 2^{e_1 + e_2}}| = |\frac{(m \pm n) 2^{e_1 + e_2 - p} \pm 2^{e_1 + e_2 - 2 p}}{mn 2^{e_1 + e_2}}| = \\\\ = |\frac{(m \pm n) 2^{- p}}{mn} \pm \frac{2^{ - 2 p}}{mn}| <= 2^{-p+2} \pm 2^{-2p+2} < 2^{-p + 3}}$$

Similarly for $k > 0$ and $p > 1$:

$$\displaylines{err_a < |(m 2^{e_1} \pm 2^{e_1 - p + k}) (n 2^{e_2} \pm 2^{e_2 - p}) - m 2^{e_1} n 2^{e_2}| =\\\\= |(m \pm n 2^k) 2^{e_1 + e_2 - p} \pm 2^{e_1 + e_2 - 2 p + k}| =\\\\= |(m 2^{-k - 1} \pm n 2^{-1} \pm 2^{-p-1}) 2^{e_1+e_2-p+k+1}| < 2 ^ {e_1 + e_2 - p + k + 1}}$$

$$\displaylines{err_r < |\frac{(m 2^{-k - 1} \pm n 2^{-1} \pm 2^{-p-1})}{mn} 2^{-p+k+1}| =\\\\= |\left(\frac{1}{n2^{k + 1}} \pm \frac{1}{2m} \pm \frac{1}{mn2^{p+1}}\right) 2^{-p+k+1}| < 2^{-p + k + 2}}$$


## Error of division

Absolute error of dividing numbers with error and $p > 3$:

$$\displaylines{err_a < |\frac {m 2^{e_1} \pm 2^{e_1 - p}} {n 2^{e_2} \pm 2^{e_2 - p}} - \frac{m 2^{e_1}}{n 2^{e_2}}| =\\\\= |\frac{(n \pm m) 2^{-p}}{n^2 \pm n2^{-p}} 2^{e_1 - e_2}| < |\left(\frac{2}{1 - 2^{-p+1}} + \frac{4}{1 - 2^{-p+1}}\right) 2^{e_1 - e_2 - p}| < 2 ^ {e_1 - e_2 - p + 3}}$$

Relative error:

$$\displaylines{err_r < |\frac{(n \pm m) 2^{-p}}{mn \pm m2^{-p}}| =\\\\= |\left(\frac{n}{m(n \pm 2^{-p})} \pm \frac{1}{n \pm 2^{-p}}\right) 2^{-p}| < \frac{2n + 1}{n - 2^{-p}} 2^{-p} < 2 ^ {- p + 3}}$$


For $k > 0$ and $p > 4$:

$$\displaylines{err_a < |\frac {m 2^{e_1} \pm 2^{e_1 - p + k}} {n 2^{e_2} \pm 2^{e_2 - p}} - \frac{m 2^{e_1}}{n 2^{e_2}}| =\\\\= |\frac{(n \pm m2^{-k}) 2^{-p + k}}{n^2 \pm n2^{-p}} 2^{e_1 - e_2}| < \left(\frac{2}{1 - 2^{-p+1}} + \frac{4 2^{-k}}{1 - 2^{-p+1}}\right) 2^{e_1 - e_2 - p + k} < 2 ^ {e_1 - e_2 - p + k + 3}}$$

$$\displaylines{err_r < \frac{2n + 2^{-k}}{n - 2^{-p}} 2^{-p+k} < 2 ^ {- p + k + 3}}$$

and $k > 0$ and $p > 2 + k$:

$$\displaylines{err_a < |\frac {m 2^{e_1} \pm 2^{e_1 - p}} {n 2^{e_2} \pm 2^{e_2 - p + k}} - \frac{m 2^{e_1}}{n 2^{e_2}}| =\\\\= |\frac{(n2^{-k} \pm m) 2^{-p + k}}{n^2 \pm n2^{-p+k}} 2^{e_1 - e_2}| < \left(\frac{2^{-k+1}}{1 - 2^{-p+1+k}} + \frac{4}{1 - 2^{-p+1+k}}\right) 2^{e_1 - e_2 - p + k} < 2 ^ {e_1 - e_2 - p + k + 3}}$$

$$\displaylines{err_r < \frac{n2^{-k+1} + 1}{n - 2^{-p+k}} 2^{-p+k} < 2 ^ {- p + k + 3}}$$

## Error of subtraction

Absolute error of subtraction of numbers with the same sign:

$$\displaylines{err_a < |(m 2^{e_1} \pm 2^{e_1 - p}) - (n 2^{e_2} \pm 2^{e_2 - p}) - (m 2^{e_1} - n 2^{e_2})| < 2 ^ {max(e_1, e_2) - p + 1}}$$

Relative error:

$$\displaylines{err_r < 2^{-p+2}}$$

Note: subtraction can cause borrow which increases relative error.

For $k > 0$:

$$\displaylines{err_a = |(m 2^{e_1} \pm 2^{e_1 - p + k}) - (n 2^{e_2} \pm 2^{e_2 - p}) - (m 2^{e_1} - n 2^{e_2})| < 2 ^ {max(e_1,e_2) - p + k + 1}}$$

$$\displaylines{err_r < 2^{-p+k+2}}$$

## Error of addition

Absolute error of addition of numbers with the same sign:

$$\displaylines{err_a < |(m 2^{e_1} \pm 2^{e_1 - p}) + (n 2^{e_2} \pm 2^{e_2 - p}) - (m 2^{e_1} + n 2^{e_2})| <= 2 ^ {max(e_1, e_2) - p + 1}}$$

Relative error:

$$\displaylines{err_r <= 2^{-p+1}}$$

For $k > 0$:

$$\displaylines{err_a = |(m 2^{e_1} \pm 2^{e_1 - p + k}) + (n 2^{e_2} \pm 2^{e_2 - p}) - (m 2^{e_1} + n 2^{e_2})| < 2 ^ {max(e_1,e_2) - p + k + 1}}$$

$$\displaylines{err_r < 2^{-p+k+1}}$$


## Error of square root

Absolute error of the square root of a number with error:

$$\displaylines{err_a < |\sqrt{m 2^e \pm 2^{e - p}} - \sqrt{m 2^e}| =\\\\= \sqrt{m 2^e}|\sqrt{\frac{m2^e \pm 2^{e-p}}{m2^e}} - 1| =\\\\= \sqrt{m 2^e}|\sqrt{1 \pm \frac{2^{-p}}{m}} - 1| < 2^{- p} \sqrt{m2^e} <= 2 ^ {\lceil{e/2}\rceil - p}}$$

(because no solution exists for $|\sqrt{1 \pm \frac{2^{-p}}{m}} - 1| >= 2^{- p}$).

Relative error:

$$\displaylines{err_r < 2^{-p}}$$

## Error of series of $\sin$, $\cos$ and $\sinh$

Error of Maclaurin series $M(x)$ of a function $f(x)$ for $x < 1$ in which absolute value of the function derivatives near 0 never exceeds 1 need to be estimated only for several first terms of the series.

Proof.

$${err < |M(m 2^e \pm 2^{e-p}) - M(m 2^e)| = 2^e |M(m \pm 2^p) - M(m)}|$$

$0.5 <= m < 1$ and $e <= 0$.

For simplisity assume $e = 0$ and the absolute value of n'th derivative is 1.

Then series look like:

$$M(x) = f(0) + x + \frac{x^2}{2!} + \frac{x^3}{3!} + ... + \frac{x^n}{n!}$$

From binomial formula $(1 + x)^n = \displaystyle\sum_{k=0}^{n}{B_k x^k}$ follows if $x = 1$:

$$2^n = \displaystyle\sum_{k=0}^{n}{B_k}\tag{1}$$

Then $(m \pm 2^{-p})^n < (1 + 2^{-p})^n = \sum{B_k 2^{-p k}}$.

Since we subtract $m^n$ from $(m \pm 2^{-p})^n$ to compute the absolute error, $k > 0$.

Then using (1) we get:

$$\sum{B_k 2^{-p k}} < \sum{B_k 2^{-p}} = 2^{n - p}$$

From Stirling's approximation $n! > (n/e) ^ n$ 
follows:

$$\frac{2^{n - p}}{n!} < 2^{n - p} \left(\frac{e}{n}\right)^n = 2^{-p} \left(\frac{2 e}{n}\right)^n$$

The residual error of the series can be received from Lagrange's error bound,
and it is smaller than $2^{-p} \left(\frac{2 e}{n + 1}\right)^{n + 1}$.

For $n$ terms of the series $err < 2^{-p} \displaystyle\sum_{k=1}^{n+1}{\left(\frac{2 e}{k}\right)^k}$.

Starting from $k = 6$ we have: $\displaystyle\sum_{k=6}^{n+1}{\left(\frac{2 e}{k}\right)^k} < 1$ and $err < 2^{-p}$.

## Error of $\arctan$, $\operatorname {arctanh}$ series.

Series:

$$\arctan(x) = x - \frac{x^3}{3} + \frac{x^5}{5} - ...$$
 
$$\operatorname {arctanh}(x) = x + \frac{x^3}{3} + \frac{x^5}{5} + ...$$

Assume, the series $x + a_3 x^3 + a_5 x^5 + ...$ is computed directly, and $x$ contains relative error less than $2^{-p}$:

$a_3$, $a_5$,... have relative error less than $2^{-p}$ since they are the result of division of 1 by the exact number 3, 5, 7,... 
and their value is smaller than 0.5 by definition.

If $x = m 2^e$, where $0.5 <= m < 1$ and $e = -3$, then absolute error relative to 1 for $\operatorname {arctanh}$:

$$\displaylines{err_a < 2^{-p-3} + 2^{-p-9+4} + 2^{-p-15+6} + ... < 2^{-p-2}}$$

or less than $2^{-p+1}$ relative to $x$. The same is true for $e < -3$.

For $\arctan$ error is $2^{-p+2}$, since the first subtraction can cause borrow.
