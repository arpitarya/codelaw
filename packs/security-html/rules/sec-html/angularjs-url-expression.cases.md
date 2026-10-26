# sec-html/angularjs-url-expression — cases

### case: href and src

```html
<div ng-app><a href="{{url}}">a</a><img src="{{ p }}" alt=""></div>
```

```expect
1:16 sec-html/angularjs-url-expression
1:41 sec-html/angularjs-url-expression
```

```fixed
<div ng-app><a ng-href="{{url}}">a</a><img ng-src="{{ p }}" alt=""></div>
```

### case: a Flask page is not AngularJS

```html
<a href="{{ url_for('x') }}">a</a>
```

```expect
(none)
```
