bike.rs is a rust implementation of minimalistic json-like language.

Some example:

```
; A comment. Document is a dictonary.
; Spaces and newlines are ignored

'hello' = T1                ; defines T1 as 'hello'
; 2128506 = mah_number        ; ...and also numbers (not implemented yet)
; true = bugs                 ; ...and booleans (not implemented yet)
'nyanyanyan' = 'nyan cat'   ; key also can be a string

('aleph' 'beth' 'gym') = list   ; a list

; a dictonary
{
  'wheelML' = name
  'be slightly crazy' = objective
  'forth, JSON' = comefrom
} = dict
```

On the name: it's all about wheel/bycicle invention.
