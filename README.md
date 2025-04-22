https://github.com/user-attachments/assets/000ffea6-db3d-4c85-b6cc-c75ee68a9e2a

# Get your search bar right

This repo is more of an implementation of my collection of ideas on what a perfect search bar should be.
I've come to get this project going after my extended frustration with the way search bars work on some websites,
namely the translation website from a company whose name starts with a G.

The most important quality for a search bar is that it should be _**permissive**_.
Namely, it must permit:
- Omtted characers
- Misvlickw
- Extera charazxcters
- Missing diacritics

On the video above, I tried to select the Hungarian language by typing out its name in the English & in the Portuguese interfaces,
the latter is included to showcase how bad search bars mishandle diacritics. As you can see, even 1 mistake is enough for
the bad search bar to start completely misunderstanding.

**It doesn't have to be like this.**

This Rust library provides:
- Basic utlities for getting similar characters, such as their versions with diacritics & their misclicked versions\*
- A search tree & an algorithm for a permissive search through the built tree that accounts for the possibility of all 4 mistakes listed above

You can depend on this library in your Rust code, copy-paste functions from it, or use it as a guideline to make search bars in your project more permissive & accessible.
