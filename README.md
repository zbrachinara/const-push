# War crimes in const rust

This repo does things in const rust which I'd really like to have. Right now it includes a
const-time arrayvec-like type, and in the future I'd like to simulate `&mut self` APIs without
actually requiring `const_mut_refs`.