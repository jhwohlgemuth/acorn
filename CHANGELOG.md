# CHANGELOG 🌱
> Wisdom is knowledge of things that don't change, intelligence is the knowledge of things that do

## Unreleased
> ⚠️ If you want to use these changes, you will need to build from source. Consult [CONTRIBUTING.md](https://code.ornl.gov/research-enablement/acorn/-/blob/main/CONTRIBUTING.md) for details.

### 🚧 In Progress

- ISBN data modeling - ([9e2f243](https://code.ornl.gov/research-enablement/acorn/commit/9e2f2435dc4fd66788b50cc830d3f46c1af43dde))

### Changed

- Replace direct Chrono usage with Jiff; public database timestamp fields now use `jiff::Timestamp`, and queue and cache intervals use `jiff::SignedDuration`.
## [0.1.49](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.48..v0.1.49) - 2025-12-24

### 🐛 Bug Fixes

- *(ci)* Combine build tasks and actually upload windows release binary - ([c59f638](https://code.ornl.gov/research-enablement/acorn/commit/c59f638fff53509f5c41902b72e5640f274907e2))
## [0.1.48](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.47..v0.1.48) - 2025-12-24

### 🐛 Bug Fixes

- *(ci)* Add powershell to rust image and fix variable in gitlab CI config - ([5e095a7](https://code.ornl.gov/research-enablement/acorn/commit/5e095a7d0ca7e8b655d5ebfd2bb6c64f3327774e))
## [0.1.47](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.46..v0.1.47) - 2025-12-24

### 🚀 Features

- Rename acorn-lib library export to acorn - ([469d75f](https://code.ornl.gov/research-enablement/acorn/commit/469d75f9b7a8bcdfa5c2067738d11039e68bd17c))
- Improve patent support in rust and python - ([eefab32](https://code.ornl.gov/research-enablement/acorn/commit/eefab322de76dfee3ecbbfa8b3917fc9a350933a))
- Add find_all to Patent impl - ([081967c](https://code.ornl.gov/research-enablement/acorn/commit/081967c5ec2a563201471de4c9a527b16eb77d6c))
- Improve patent regex parsing - ([e7cbb41](https://code.ornl.gov/research-enablement/acorn/commit/e7cbb4139d86a6fcc592bfd1d9de042911c5356b))
- Create patent module - ([e12fe35](https://code.ornl.gov/research-enablement/acorn/commit/e12fe35e85e91e6ef73c75d8aefa131afd467c57))
- Add DOI find_all to rust and python packages - ([a99b7b7](https://code.ornl.gov/research-enablement/acorn/commit/a99b7b7d75f1d74e5a749fe41d3455c83e5967d7))
- Initial addition of ASPECT data to powerpoint interpolation - ([1bf33b2](https://code.ornl.gov/research-enablement/acorn/commit/1bf33b2bb1670bcd34eabd4e998864ea914b68b5))
- Refine PowerPoint interpolation capability - ([7e87519](https://code.ornl.gov/research-enablement/acorn/commit/7e875193f4a701a496a22c9e15d810b0591073d6))
- Standardize use of ToAbsoluteString trait - ([a1b2d0c](https://code.ornl.gov/research-enablement/acorn/commit/a1b2d0c79fdaae19b6368604287deef1cccb178e))
- Remove export Size and Target options (gate on file format and change reference for highlights) - ([69600ee](https://code.ornl.gov/research-enablement/acorn/commit/69600ee1a9a8c720e0da44f010e36470d5c79bf2))
- Rename efficacy to agency for understanding and meaning - ([c95e9a8](https://code.ornl.gov/research-enablement/acorn/commit/c95e9a8dc735c79db85961199294cdf7ba6f3830))
- Refine RA format function - ([c973d77](https://code.ornl.gov/research-enablement/acorn/commit/c973d77ca6cadb4ff1edccbaeb4595ec379c4f16))
- Remove FuzzyValue enum - ([349e004](https://code.ornl.gov/research-enablement/acorn/commit/349e0042b4340fb10cd8d36c91ccc093158c59ca))
- Add from_technology method for License impl - ([2eb81fe](https://code.ornl.gov/research-enablement/acorn/commit/2eb81feac1ce50b9b7af4275b606db4bd9fe3f1f))
- Add enhanced License handling capability; Make technology a valid CSV - ([e859fef](https://code.ornl.gov/research-enablement/acorn/commit/e859fef6cebef19b4a837255efd65305001f88ad))
- First wasi build of acorn-lib with no errors or issues - ([9afa57d](https://code.ornl.gov/research-enablement/acorn/commit/9afa57ddb6ea28fdf91a29df110d3490c82ff0d8))
- More progress toward no-default build - ([0150619](https://code.ornl.gov/research-enablement/acorn/commit/01506195117c565a8053fa0255e681c57e9ea445))

### 🐛 Bug Fixes

- *(docs)* Add carriage return before code example - ([48df3c9](https://code.ornl.gov/research-enablement/acorn/commit/48df3c9e81bc9931d5791adc52949f14e6e20d35))
- *(tests)* Update doctests to pass and add no-default test to ci task - ([cd2e89e](https://code.ornl.gov/research-enablement/acorn/commit/cd2e89e01d6347f3a3dca838def4aab366d3e162))
- Include default hasher feature for hashbrown; use hashbrown for std and no-std builds - ([b267a1b](https://code.ornl.gov/research-enablement/acorn/commit/b267a1b58490f7af53dd0054d61bdb5fb7d80337))

### 📚 Documentation

- Update overview image - ([28ae923](https://code.ornl.gov/research-enablement/acorn/commit/28ae92353bc6708b4960bd5ffb89b8fb70b6e772))
- Update shell examples for export command in book - ([b79d91a](https://code.ornl.gov/research-enablement/acorn/commit/b79d91a27c7993760e798663d51706600539eae6))
- Add more clarity to agency page - ([9072cb5](https://code.ornl.gov/research-enablement/acorn/commit/9072cb5078c74282d40efb86f427978202c28994))
- Build out agency page - ([533a1f9](https://code.ornl.gov/research-enablement/acorn/commit/533a1f9a53da899d1b67ec8062bb299b99934f5d))
- Add words to HMT page - ([96e1908](https://code.ornl.gov/research-enablement/acorn/commit/96e1908ca1eb6f81d96ecc433888ca9fcf6e68c2))
- Refactor structure; improve aspect docs; various improvements - ([54edbf7](https://code.ornl.gov/research-enablement/acorn/commit/54edbf72a7e3eadb72002078cc57675486867219))

### ⚙️ Miscellaneous Tasks

- Add windows build to CI release jobs - ([0afd433](https://code.ornl.gov/research-enablement/acorn/commit/0afd433e6c556695a97728163dd5c77d43764374))
- Initial prep for adding windows release - ([2e89cb8](https://code.ornl.gov/research-enablement/acorn/commit/2e89cb89757386cf6e5791aa98d93e2ab7ff800a))

### 🤓 Developer Experience

- Add windows build task - ([14d0009](https://code.ornl.gov/research-enablement/acorn/commit/14d000903b6fda0ab953b6270810152207d8479b))
- Add reference to WARP file to agent files - ([43f1f4c](https://code.ornl.gov/research-enablement/acorn/commit/43f1f4ce3fa043738512dfac6a29a29f29c80565))
- Add agent configuration files - ([f3795db](https://code.ornl.gov/research-enablement/acorn/commit/f3795dbf18c3d5e0c7ca8383622afc631a07c3d6))
## [0.1.46](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.45..v0.1.46) - 2025-12-10

### 🚀 Features

- Remove PathBuf from util module (moved to io module) - ([0c2df3f](https://code.ornl.gov/research-enablement/acorn/commit/0c2df3f803963608516daed51416c03bf8fd17ec))
- Add portability to aspect framework - ([6e1b5a2](https://code.ornl.gov/research-enablement/acorn/commit/6e1b5a23d80c58dddf97719a19b373e34d2aba00))
- Move more functions that depend on PathBuf into io module - ([c894d40](https://code.ornl.gov/research-enablement/acorn/commit/c894d40472b69ec6257aa098b31157c8dc016c6a))
- Add technologies from midnight blue - ([2dca698](https://code.ornl.gov/research-enablement/acorn/commit/2dca6984c3ecfcabb6eea530aeaf4987cd064a6c))

### 🐛 Bug Fixes

- *(ci)* Remove manual triggers from jobs - ([d79fc44](https://code.ornl.gov/research-enablement/acorn/commit/d79fc446bfbde3cce8ff0488ff1ec59a9187c1c1))
- *(docs)* Add aspect portability to summary - ([151fb4e](https://code.ornl.gov/research-enablement/acorn/commit/151fb4e3e6b40f5e28479a17bf2b49b726d6c7d4))

### 📚 Documentation

- Add image and some links to motivation - ([40a149d](https://code.ornl.gov/research-enablement/acorn/commit/40a149dbb851dc780dc46d1adedcac4b0fc01a53))
- Add motivation and build out aspect overview - ([f5078f5](https://code.ornl.gov/research-enablement/acorn/commit/f5078f5fd8ac79f7ec7da32024c6bde452f45ede))
- Add maturity page to book - ([4f1c04a](https://code.ornl.gov/research-enablement/acorn/commit/4f1c04a75dc3a74dabf3ce19990a9861518b9ee7))
- Add badges to book - ([cd06df0](https://code.ornl.gov/research-enablement/acorn/commit/cd06df06b270ef7d8733f54f8b5706ef4b9b485b))
- Add custom 404 page for book - ([8115e8b](https://code.ornl.gov/research-enablement/acorn/commit/8115e8bb712782dbfae141c95c46303467dbf5db))
- Improve readability of book and add more sections - ([58f7f9e](https://code.ornl.gov/research-enablement/acorn/commit/58f7f9ea1d0abf055790a7d512fea5c41124f3e5))
- Update acorn banner with new font - ([f2a9c05](https://code.ornl.gov/research-enablement/acorn/commit/f2a9c058e84280099d041576b46e4e0ed3910817))
- Add more words to book overview - ([019df6d](https://code.ornl.gov/research-enablement/acorn/commit/019df6d1441de8942d493c2fa77cba968d082a2b))
- Initial addition of ASPECT to book - ([b43cc4a](https://code.ornl.gov/research-enablement/acorn/commit/b43cc4af412197727f2dd3da2189290e76f6b18c))
- Update changelog - ([87b3109](https://code.ornl.gov/research-enablement/acorn/commit/87b31090833ec264b3f5e2ff0018e6b2bccd2088))
## [0.1.45](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.43..v0.1.45) - 2025-12-05

### 🚀 Features

- Add model and llm structs to aspect - ([2322569](https://code.ornl.gov/research-enablement/acorn/commit/2322569b08b24b3ad1f729df13ea4c6016726bdd))
- Integrate aspect attribute in RAD schema - ([a4ca38b](https://code.ornl.gov/research-enablement/acorn/commit/a4ca38b2f88ed0aa18c6c6a4d4b306df279e1ffb))
- Initial addition of ASPECT schema - ([0aa3a2a](https://code.ornl.gov/research-enablement/acorn/commit/0aa3a2a361a86d8e058eee39d0b1d6ff3cc42f35))
- Move RAD metadata struct to research_activity module - ([fd8636f](https://code.ornl.gov/research-enablement/acorn/commit/fd8636f4e8d0723fcf49251b8fa51bfa53d46de9))

### 🐛 Bug Fixes

- *(ci)* Move docs stage back to end - ([7bda57b](https://code.ornl.gov/research-enablement/acorn/commit/7bda57b358bdd6d92bc8ec53bf1e8ff662e05f0f))
- *(tests)* Update import in python package and remove trl attribute from test data - ([705f641](https://code.ornl.gov/research-enablement/acorn/commit/705f6418cd2a545a1e2fcb67632b0cfc791d543a))
- Re-add LLM struct to aspect - ([3921c79](https://code.ornl.gov/research-enablement/acorn/commit/3921c795807eba4d68ba99a3fbadea9838ab96cc))

### 📚 Documentation

- Add bottom margin to banner - ([f0f1b64](https://code.ornl.gov/research-enablement/acorn/commit/f0f1b64b1f015ed10b3d1551ebd97ecaeceeb1bc))
- Add light and dark mode support for real hopefully - ([aa2aed1](https://code.ornl.gov/research-enablement/acorn/commit/aa2aed1499bef3b85726a0bd3fed3cc20309f3b3))
- Add light and dark mode support - ([43e2e2e](https://code.ornl.gov/research-enablement/acorn/commit/43e2e2e70eef4b26553cd9b4cd7feadca0c80b69))
- Add ACORN banner to README - ([5930805](https://code.ornl.gov/research-enablement/acorn/commit/59308055ac8842760f4e639f6a5ebb5d17311f06))
- Generate changelog - ([1905ace](https://code.ornl.gov/research-enablement/acorn/commit/1905ace81ac17b149aad9fbd4977c2f9af31c1d2))

### 🚧 In Progress

- *(docs)* Try to enable light and dark mode support - ([9c3f750](https://code.ornl.gov/research-enablement/acorn/commit/9c3f7508acf409812b4355515083b1679ae3ffc3))
## [0.1.43](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.42..v0.1.43) - 2025-11-27

### 🚀 Features

- Make some functions less public behind pub(crate) directives - ([33d5804](https://code.ornl.gov/research-enablement/acorn/commit/33d5804000590d630240d260696c12d84f0dc304))
- Move ResearchActivity code to separate module to improve modularity and readability - ([4a3da19](https://code.ornl.gov/research-enablement/acorn/commit/4a3da196a7762908b87910bbf8f623a51eddd7b8))
- Improve markdown conversion and linked data context handling with traits - ([8667e6f](https://code.ornl.gov/research-enablement/acorn/commit/8667e6f1e00bba0fa078565ddeebf56d26e49681))
- Refine and fix JSON schema output - ([1aea369](https://code.ornl.gov/research-enablement/acorn/commit/1aea36910d06cc2d76cb00885bbce8c87d17b6ff))
- Add toolchain file - ([d1fc837](https://code.ornl.gov/research-enablement/acorn/commit/d1fc8379d05033c602d154d973397b82f6d275a0))
- Add image assets to book - ([7f71442](https://code.ornl.gov/research-enablement/acorn/commit/7f71442ed6c81b8eb7b6120d55d400240d2499b5))
- Rename buckets to config ISO of combined application configuration - ([de15aa5](https://code.ornl.gov/research-enablement/acorn/commit/de15aa5544a0700e10e518ecb114088dd7ed3c7a))
- Move configuration code to acorn-cli and set stage for application configuration - ([1f0f31b](https://code.ornl.gov/research-enablement/acorn/commit/1f0f31bdd828003866b05ed59880d4ef0a2720d3))
- Move which and its deps into io module - ([7054b9f](https://code.ornl.gov/research-enablement/acorn/commit/7054b9f5559189865f8e2c9d7ee593f83d4bea98))
- Prepare initial book documentation requirements - ([a5d853a](https://code.ornl.gov/research-enablement/acorn/commit/a5d853adc6315ff73c45bc0d68854658a36992dd))
- Add DTRA to sponsors - ([bebd33e](https://code.ornl.gov/research-enablement/acorn/commit/bebd33e7fb119d437e7fc1b71e88cce4473d9d42))
- Add vale and acorn to bucket runner image - ([e3cb270](https://code.ornl.gov/research-enablement/acorn/commit/e3cb27068cc5ca9881a8c9b695519231ba2e8b1e))

### 🐛 Bug Fixes

- *(ci)* Add mdbook dependencies - ([da097fe](https://code.ornl.gov/research-enablement/acorn/commit/da097fe108ee04de278c9dd72c5f65956cb70434))
- *(ci)* Make publish jobs manual - ([7887fa1](https://code.ornl.gov/research-enablement/acorn/commit/7887fa12fba5122c80572c66a999495ddce274ed))
- *(ci)* Make release job optional and move book job to end - ([de73bca](https://code.ornl.gov/research-enablement/acorn/commit/de73bca9717fdfded8a88cf2fd30205d2ef2f52c))
- *(ci)* Make book job optional again - ([825639f](https://code.ornl.gov/research-enablement/acorn/commit/825639ff5cf73b15d106cc4a75094af8236ce8bf))
- *(ci)* Make docs stage first again - ([37814f6](https://code.ornl.gov/research-enablement/acorn/commit/37814f655dcc7c349143437f5a3cf355094f5667))
- *(ci)* Install mdbook - ([8653f23](https://code.ornl.gov/research-enablement/acorn/commit/8653f23386a3de81d7793be3008b336fa2faa495))
- *(ci)* Make docs stage first - ([1119d72](https://code.ornl.gov/research-enablement/acorn/commit/1119d720d20fba0c1f48deabc824fbe4b236bc99))
- *(ci)* Remove rules from book job - ([fce6b7c](https://code.ornl.gov/research-enablement/acorn/commit/fce6b7c019901a993e016f2822d05edd1078c38e))
- *(ci)* Make book job manual - ([850f495](https://code.ornl.gov/research-enablement/acorn/commit/850f4956e55a1f0adb107a3d4160787daca872eb))
- *(ci)* Make book job optional - ([63b824c](https://code.ornl.gov/research-enablement/acorn/commit/63b824cd0b9a353218f3150c118b9ccfbc0e2c7e))
- Add serde derive to service point response struct - ([911442b](https://code.ornl.gov/research-enablement/acorn/commit/911442b5fdc4b7107425de84ec290263d2424b10))
- Link subcommand should save jsonld file - ([3cdd267](https://code.ornl.gov/research-enablement/acorn/commit/3cdd267efbabc348db8931c62eb3b3a87223347d))
- Pin FROM image for bucket Dockerfile - ([141fc28](https://code.ornl.gov/research-enablement/acorn/commit/141fc2872ba7cbd077612a6bf95c17c4425daeaa))

### 📚 Documentation

- Generate changelog - ([ea6bf8f](https://code.ornl.gov/research-enablement/acorn/commit/ea6bf8f3daa3ba620fb679f4b5ae090f3af44c55))
- Update acorn banner again - ([0020f9b](https://code.ornl.gov/research-enablement/acorn/commit/0020f9b523a21d3a6d2177b4ff5fa44c580bda3c))
- Update acorn banner - ([acaa09b](https://code.ornl.gov/research-enablement/acorn/commit/acaa09b921ddea85ede674faa827d85abba38292))
- Update link command page - ([7655077](https://code.ornl.gov/research-enablement/acorn/commit/76550778bef2096a545ebb0f2cb589cf1a438d88))
- Add ACORN banner - ([30fb344](https://code.ornl.gov/research-enablement/acorn/commit/30fb344e788d001a57e26b1044c60a7c9af95503))
- Update export command page - ([85232c5](https://code.ornl.gov/research-enablement/acorn/commit/85232c554300adfce565231afab4b9224a164ad7))
- Update format command page - ([c2ce056](https://code.ornl.gov/research-enablement/acorn/commit/c2ce056e81bc571b88af11c03c7cea7e625f8a3a))
- Update download command page in book; Add title to all command pages - ([d78e87a](https://code.ornl.gov/research-enablement/acorn/commit/d78e87a1614263406f05e70cd967d4095c70e5b0))
- Update check command help in book - ([a632f25](https://code.ornl.gov/research-enablement/acorn/commit/a632f2515edeb9d4f97b8a64b5c3a9eb91f125bf))
- Populate page for check command - ([29b6f86](https://code.ornl.gov/research-enablement/acorn/commit/29b6f8670f0f2a7d5d3e238a9bea4115c6b95405))
- Add book to CI build - ([b22bd12](https://code.ornl.gov/research-enablement/acorn/commit/b22bd12b111f3b6346bf34258e3213b71d54ca9d))
- Update acorn readme ascii art - ([6c649d1](https://code.ornl.gov/research-enablement/acorn/commit/6c649d17049b1bc87158d0886a3acf9ab9e464fd))
- Cleanup introduction a bit - ([3183213](https://code.ornl.gov/research-enablement/acorn/commit/31832131d5ea3c09017e9ba8dd527221b8278b98))

### 🚧 In Progress

- *(ci)* Make docs stage first again - ([5d10672](https://code.ornl.gov/research-enablement/acorn/commit/5d106724747d06751d5276839bc0637a753a8836))
- *(ci)* Try copying test rules to make book job manual - ([fbb8059](https://code.ornl.gov/research-enablement/acorn/commit/fbb80593cad26e72e0c15497636550593fe56097))
## [0.1.42](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.41..v0.1.42) - 2025-11-17

### 🚀 Features

- Make check command respect --quiet - ([c22f773](https://code.ornl.gov/research-enablement/acorn/commit/c22f773708699c1a0923b146367f85e5f47facf8))

### 🐛 Bug Fixes

- *(ci)* Remove arguments from pixi run command - ([575b35f](https://code.ornl.gov/research-enablement/acorn/commit/575b35fbc75cc9dbe87e9b0c39ef20c6b86fbaf3))
- *(ci)* Add build-essential to packages on pixi image - ([0a1117f](https://code.ornl.gov/research-enablement/acorn/commit/0a1117f0efa55e81ffd940e7558455c8dceb1129))
- *(ci)* Source env file to make uv available in python test job - ([7e1d79a](https://code.ornl.gov/research-enablement/acorn/commit/7e1d79ac392b5ca8458f24ce3381ea307c018d1d))
- *(ci)* Update python test job to install uv - ([fea4a2c](https://code.ornl.gov/research-enablement/acorn/commit/fea4a2c6dd140bcbb07422522fd9f13c325bc94b))

### ⚙️ Miscellaneous Tasks

- Refactor out default settings from CI config - ([f6460b8](https://code.ornl.gov/research-enablement/acorn/commit/f6460b85c896faf830ecc5588c8dfa36eff6358c))
- Use custom rust docker image to speed up CI jobs - ([5e4ade0](https://code.ornl.gov/research-enablement/acorn/commit/5e4ade0660452b173762b38ad19fd802a94226db))
## [0.1.41](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.40..v0.1.41) - 2025-11-16

### 🚀 Features

- Clear lint issue; Create rust image Dockerfile; Add manual trigger to job - ([0a2cf8d](https://code.ornl.gov/research-enablement/acorn/commit/0a2cf8d130fd453ae3aa2bd86b14dd05e1ce69b9))
## [0.1.40](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.39..v0.1.40) - 2025-11-16

### 🐛 Bug Fixes

- *(ci)* Fix manual trigger for publish-book - ([19539af](https://code.ornl.gov/research-enablement/acorn/commit/19539af6db6c09f861f4c3c1781678698e28cf42))
- *(ci)* Add quotes to publish-crates code; fix test manual job triggers - ([f82aba2](https://code.ornl.gov/research-enablement/acorn/commit/f82aba244926090cbe8732119f22b3ff00f056aa))
## [0.1.39](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.38..v0.1.39) - 2025-11-16

### 🐛 Bug Fixes

- *(ci)* Update rule and task for publish-book job - ([1f7a7f4](https://code.ornl.gov/research-enablement/acorn/commit/1f7a7f4b75e785d837013a6c2dad50074229e808))
- *(ci)* Only publish-crates runs based on version tags - ([c983c21](https://code.ornl.gov/research-enablement/acorn/commit/c983c2182cdf2efa084111ebdb12bb5dfbd3f1b2))
- Gate unix imports behind cfg - ([2dbd57c](https://code.ornl.gov/research-enablement/acorn/commit/2dbd57cede2efba5980ce89ff6f691b4746047d9))

### ⚙️ Miscellaneous Tasks

- Allow manual run for tests - ([83cf746](https://code.ornl.gov/research-enablement/acorn/commit/83cf746c4ef0b33fbc2a2e8302d0fb56a0219646))
## [0.1.38](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.35..v0.1.38) - 2025-11-16

### 🚀 Features

- Improve RAID PID functionality - ([baabf47](https://code.ornl.gov/research-enablement/acorn/commit/baabf47344afe538a6c7175dd3853d8bdf1c89e8))
- Add test for RAID functions - ([ab10188](https://code.ornl.gov/research-enablement/acorn/commit/ab10188463bb6998c5576d3b8354531b9bd87544))
- Add RAiDs to pid module and create raid service point data structures - ([6fa8850](https://code.ornl.gov/research-enablement/acorn/commit/6fa8850aa104ab7e34c01e1f02643b935f0e87f1))
- Move citeas module to io module - ([a043e87](https://code.ornl.gov/research-enablement/acorn/commit/a043e874eb328c9dfe9ee9582b721feffeebdc5c))
- Move constants module under util - ([0f1fac8](https://code.ornl.gov/research-enablement/acorn/commit/0f1fac8b1649a729b4fe27e130a726b5eb428acf))
- Gate duct dependency in util and move io module out of util - ([e8018b0](https://code.ornl.gov/research-enablement/acorn/commit/e8018b06d5e57a4021be9bee76ecfd475fb3f924))
- Remove more prelude imports from util by moving functions and re-writing file_extension function - ([f3ef144](https://code.ornl.gov/research-enablement/acorn/commit/f3ef144545be2443e1503279cf349e89ed3532dc))
- Initial link command functionality - ([b8b7066](https://code.ornl.gov/research-enablement/acorn/commit/b8b7066d8d1e7dee1301d9605804d22ccdf32b14))
- Isolate reqwest behind std feature; make std dependencies optional and gated by features (prelude might be last obstacle) - ([025d3f5](https://code.ornl.gov/research-enablement/acorn/commit/025d3f54673bf4b2630df7bc007e5fbe24608cf2))
- Isolate tokio runtime dependency - ([910374e](https://code.ornl.gov/research-enablement/acorn/commit/910374e26c99188058c9bbd62aa2bc5aba347c25))
- Refine feature partitions - ([18dd457](https://code.ornl.gov/research-enablement/acorn/commit/18dd457b52e2310cb3da97e3b3938eb1b62e1484))
- Add linked data context for Metadata and ResearchActivity (still need to create acorn link command) - ([e2f861f](https://code.ornl.gov/research-enablement/acorn/commit/e2f861f02104eff44b604b35dee6808207687b86))

### 🐛 Bug Fixes

- *(ci)* Bump version to test CI - ([be690e0](https://code.ornl.gov/research-enablement/acorn/commit/be690e02cdd9c2082e7cca1620339099310beee5))
- *(ci)* Update prepare-environment job rules - ([9892bd6](https://code.ornl.gov/research-enablement/acorn/commit/9892bd656820e8f216a24a7907610d8d6b855a61))
- *(ci)* Need to run release jobs to create version tags - ([75333dc](https://code.ornl.gov/research-enablement/acorn/commit/75333dccc1dd3fad9f6d424ec530410353c75ca4))
- *(ci)* Remove curly brackets from job if conditions - ([bab34cb](https://code.ornl.gov/research-enablement/acorn/commit/bab34cb9606e9a3f972e1dc0e538324ad6ea1e5f))
- *(tests)* Ignore problematic test - ([c278650](https://code.ornl.gov/research-enablement/acorn/commit/c27865025761a2b5a8567c273d49e294c38172cc))

### 🧪 Testing

- Add test for PID trait - ([4b840f5](https://code.ornl.gov/research-enablement/acorn/commit/4b840f5f191dd931cd835c0c794db4176db752c0))

### ⚙️ Miscellaneous Tasks

- Only run prepare-environment on version tag - ([f0a48bc](https://code.ornl.gov/research-enablement/acorn/commit/f0a48bc1fb34b3e4eaf7f5c5939d008a7e58f1c3))
- Build release when commit has version tag - ([78d30ad](https://code.ornl.gov/research-enablement/acorn/commit/78d30ad118acac9f64785c5e57067bbcad1e6b68))
- Use DEFAULT_TARGET in jobs - ([2a75023](https://code.ornl.gov/research-enablement/acorn/commit/2a750236e7e71eceef0b498901a2bf7bb793961e))
## [0.1.35](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.34..v0.1.35) - 2025-11-09

### 🐛 Bug Fixes

- Remove wayward line from CI config - ([6474a9e](https://code.ornl.gov/research-enablement/acorn/commit/6474a9e8b9d912a5fafe0df0ec7f627442e9fd9a))
## [0.1.34](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.33..v0.1.34) - 2025-11-09

### 🚀 Features

- Add linked data context for ContactPoint - ([5266a0e](https://code.ornl.gov/research-enablement/acorn/commit/5266a0e21b46c307c3ff9f53f534335eab6db0de))
- Add meta.status attribute (default value = active) - ([384fda0](https://code.ornl.gov/research-enablement/acorn/commit/384fda09e3ef9ce153ace269d86a26a0f90da0f5))
- Remove exit from acorn-lib and prelude module; make exitcode an acorn-cli only dependency - ([b2cc81b](https://code.ornl.gov/research-enablement/acorn/commit/b2cc81b6ce161dcd29833704a41b828c67e1f54d))
- Create bucket module and improve release error handling - ([a7fbabf](https://code.ornl.gov/research-enablement/acorn/commit/a7fbabf7eceb1c061f84beb2de53fa55600f3cf6))
- Enable std feature in acorn-cli - ([b743820](https://code.ornl.gov/research-enablement/acorn/commit/b743820a4505949dd60dd4d45bd5c9edf2b3fe54))
- Gate util::io behind std feature - ([c05385a](https://code.ornl.gov/research-enablement/acorn/commit/c05385a43e6f95ec2999f3d4b36d36fd1c8bbdb4))
- Move more util functions to io module - ([a19c5c5](https://code.ornl.gov/research-enablement/acorn/commit/a19c5c52c64a539df37862fe4d7719e6618ab570))
- Improve error handling and remove explicit io dependency from git utilities - ([f4ced2c](https://code.ornl.gov/research-enablement/acorn/commit/f4ced2c44c7ebbd9f515ee20137d89f1a274d612))
- Create util::io module to group io functions - ([e03aefd](https://code.ornl.gov/research-enablement/acorn/commit/e03aefd757221318e5e7d8604d0ae111aefbbdea))
- Refactor lib and cli to have improved modularity and more isolated dependencies - ([450ac18](https://code.ornl.gov/research-enablement/acorn/commit/450ac188787f7846d58c93d7e9da7b4036fced14))
- Refactor readability code to be more modular - ([c068ffe](https://code.ornl.gov/research-enablement/acorn/commit/c068ffe7e5b834d860bea6a8673cf73bfc4dcfeb))
- Remove wildcard imports of external modules - ([0852a85](https://code.ornl.gov/research-enablement/acorn/commit/0852a85ced91ebedd4e4f230e7af5268ee80b6a1))
- Remove wildcard imports of internal modules - ([037b8de](https://code.ornl.gov/research-enablement/acorn/commit/037b8de8ee3d695fe8a1d839220cabb2d461ef5a))
- Add udeps to audit task and CECILL-B license (in prep of eventually adding sophia crate) - ([c2f5b2c](https://code.ornl.gov/research-enablement/acorn/commit/c2f5b2cf8664fd3bf1e5a763cb3d7b4b7c7ffdea))
- Integrate cargo-deny into make task - ([4498acf](https://code.ornl.gov/research-enablement/acorn/commit/4498acfb4c7c98f25059f9f16f40f74cac841dae))
- Add cargo-deny configuration - ([81896bd](https://code.ornl.gov/research-enablement/acorn/commit/81896bd5d373683e347dd85011163c62816e1f8e))
- Remove (currently) unused dependencies from acorn-lib - ([f891236](https://code.ornl.gov/research-enablement/acorn/commit/f891236000131575db42448afb9951fe9c23b6c2))
- Group inline std imports in prelude - ([fac59f1](https://code.ornl.gov/research-enablement/acorn/commit/fac59f1412543585ed20096281effb2ad90c9d5a))
- Create prelude module to isolate std library usage and eventually enable no_std support - ([cd9a6cd](https://code.ornl.gov/research-enablement/acorn/commit/cd9a6cd2a6c38665013b3e522c16cca789a88ada))
- Implement NCDA with limited tests - ([07c47ed](https://code.ornl.gov/research-enablement/acorn/commit/07c47edc2150c30cd8a29d89db985fe2552afd22))
- Improve check_digit trait integration and ORCID implementation - ([abee417](https://code.ornl.gov/research-enablement/acorn/commit/abee4177666ac2d1e81922630feeb615f7b6aba9))
- Refine trait usage in PID data modeling - ([6f435a7](https://code.ornl.gov/research-enablement/acorn/commit/6f435a7bf355599532e7b36828c8772898a43111))
- Improve publish task and documentation - ([41877ae](https://code.ornl.gov/research-enablement/acorn/commit/41877ae07450cc685d56b8bd5c5e83a322dfb64c))
- Disable default features in more lib dependencies - ([31c5b9d](https://code.ornl.gov/research-enablement/acorn/commit/31c5b9d84f6072e1874f844125f0575310ab0bbb))
- Enforce new clippy lint rules to improve no_std posture - ([e650bdf](https://code.ornl.gov/research-enablement/acorn/commit/e650bdf57d6b3fbf951b935adcd69a221fc89d14))
- Refine acorn-lib dependency granularity and overtness - ([6589a0a](https://code.ornl.gov/research-enablement/acorn/commit/6589a0ace94d6494f22ff6a091e1368c662d0ad7))
- Update acorn-lib dependencies - ([5b60f2b](https://code.ornl.gov/research-enablement/acorn/commit/5b60f2bf26de0775102c23326c8c36202d6439ed))
- Refine workspace dependencies some more - ([6f624ff](https://code.ornl.gov/research-enablement/acorn/commit/6f624ff01d218d46809cc547a5920c5ead183692))
- Add to_strings method to vector lists of paths - ([482061c](https://code.ornl.gov/research-enablement/acorn/commit/482061c9ffe17d003a4ea6febb1e017c879fb3db))
- Update workspace dependencies and set default features to false - ([71af77a](https://code.ornl.gov/research-enablement/acorn/commit/71af77a9e07c9fb4bf210c302ae2f168a21437bf))
- Add features and simplify util dependencies - ([46d514f](https://code.ornl.gov/research-enablement/acorn/commit/46d514f6f6732e69e96c5cf96097e104810cf926))
- Refine acorn-py workspace configuration and update pyo3 dependency - ([5ef3ea5](https://code.ornl.gov/research-enablement/acorn/commit/5ef3ea5b0a830138112081c7684c77a7c5028589))
- Add ARK support to validation and Python API - ([55dc578](https://code.ornl.gov/research-enablement/acorn/commit/55dc578655bd5764300d9fa37dc4cacfd23db225))
- Add support for working with and validating ARK values - ([ce3d8e8](https://code.ornl.gov/research-enablement/acorn/commit/ce3d8e8da7929d53b4eb9cfd35b221a4013bde82))
- Make analyzer traits more flexible by removing explicit mentions of Vale and ValeConfig - ([c4dcaad](https://code.ornl.gov/research-enablement/acorn/commit/c4dcaad2ec2f2489a9e6f5c2ccb28e418b9946ef))
- Improve code for vale resolution - ([74c515f](https://code.ornl.gov/research-enablement/acorn/commit/74c515f60f29b8cfe3caaf9395720b9f3353c4ae))
- Improve flexibility and readability of vale download capability - ([2230e98](https://code.ornl.gov/research-enablement/acorn/commit/2230e981aabb98f88ab8b1f8770f97d7d47846ea))
- Refine extract_zip flexibility - ([4500504](https://code.ornl.gov/research-enablement/acorn/commit/45005041f95db8ac71596b34967c213e1d81d6c8))
- Parameterize all acorn instances in ci config - ([da8aee8](https://code.ornl.gov/research-enablement/acorn/commit/da8aee8e0bf9720303e33446adc0cfe72bd257d7))

### 🐛 Bug Fixes

- *(ci)* Install clippy for CI run - ([8c51dbf](https://code.ornl.gov/research-enablement/acorn/commit/8c51dbf2eec27066b158c4a9a45e703ba0d32421))
- *(tests)* Update ARK values used in Python tests - ([f011b11](https://code.ornl.gov/research-enablement/acorn/commit/f011b11365a1416e550dea94592919d8705fcae2))
- Add zip feature to restore functionality - ([c258e68](https://code.ornl.gov/research-enablement/acorn/commit/c258e68f34868302c53e484c872049ae4bd38dee))
- Only try to copy files when repository is local - ([7073a24](https://code.ornl.gov/research-enablement/acorn/commit/7073a243117ae661689413a281ff440b2463d981))
- Update download command to support local and remote URIs - ([021f079](https://code.ornl.gov/research-enablement/acorn/commit/021f0791968f5a3883c3ecb3587f88749a062fcd))
- Pin tracing-subscriber to ensure ANSI support (until fixed) - ([87cd2e0](https://code.ornl.gov/research-enablement/acorn/commit/87cd2e0a4e299e7f5f99bce9238e1289f58dac68))
- Add necessary features to clap and percy-dom - ([1e8818b](https://code.ornl.gov/research-enablement/acorn/commit/1e8818bf8219c59502b652190030b25232e7adf3))

### 📚 Documentation

- Add examples for running acorn commands from source with cargo - ([a7ffc3e](https://code.ornl.gov/research-enablement/acorn/commit/a7ffc3e59aadf729c36168534185a1d48ce7c49c))
- Update pre-compiled binary install instructions - ([234869c](https://code.ornl.gov/research-enablement/acorn/commit/234869c3504aa24247673a08e815ffb2532e0c8e))

### 🧪 Testing

- Rename bucket config with local bucket - ([1257376](https://code.ornl.gov/research-enablement/acorn/commit/125737639eb56d9508c8bc72e09998eba8792002))
- Add similar-asserts (might eventually fully replace pretty-assertions) - ([c2160ff](https://code.ornl.gov/research-enablement/acorn/commit/c2160ff724529fc0f86224e1e639bea9b2dd05e0))
- Add unit tests from xylem - ([3d8b8c6](https://code.ornl.gov/research-enablement/acorn/commit/3d8b8c68eb06177f81fdf3b0f342e4729af4ce77))
- Add test for is_ark error message - ([31cc6a3](https://code.ornl.gov/research-enablement/acorn/commit/31cc6a377e7dbe011cca6a033d6480305c0bd7db))

### ⚙️ Miscellaneous Tasks

- Add publish-crates job; allow failure on publish-docs job - ([9055df1](https://code.ornl.gov/research-enablement/acorn/commit/9055df135888a66de61cd429d1405ace5a059940))

### 🚧 In Progress

- Initial addition of ARK structure - ([b8a9734](https://code.ornl.gov/research-enablement/acorn/commit/b8a973475efe7d8b2d3e023a54e24e202f5fbb89))

### 🤓 Developer Experience

- Refine python dev tasks ISO of adding more sophisticated publishing - ([8c76bfd](https://code.ornl.gov/research-enablement/acorn/commit/8c76bfd8f5e28434e71d3971903c1cd1c49c734c))
## [0.1.33](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.32..v0.1.33) - 2025-10-23

### 🐛 Bug Fixes

- *(ci)* Use inherited variables where needed - ([6514061](https://code.ornl.gov/research-enablement/acorn/commit/6514061ee4d9e9d75d7cf3416b9fc3f0e1e9c378))

### 🚧 In Progress

- Bump version to test musl build - ([62d58ea](https://code.ornl.gov/research-enablement/acorn/commit/62d58ea8a04aceeaa9abadf754bc5d42b8388e3b))
## [0.1.32](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.31..v0.1.32) - 2025-10-23

### 🚀 Features

- Make default target MUSL; Update release profile settings; Pin ci image to bookworm - ([ca81f8d](https://code.ornl.gov/research-enablement/acorn/commit/ca81f8d22d30838002819179fc6e1e1ba63fbb47))
- Refine executable check on windows - ([765e357](https://code.ornl.gov/research-enablement/acorn/commit/765e3578a55f3c137b858e1ac42ab9f2af17fadc))
- Vale release zip extraction works on windows - ([f8d7093](https://code.ornl.gov/research-enablement/acorn/commit/f8d709366494200b8dc40dc35eaa9a9cb111285c))
- Add initial capability to download vale on Windows - ([3974fad](https://code.ornl.gov/research-enablement/acorn/commit/3974fad97d950200380da2ee7a4a71fd846497fd))
- Add skip-verify-checksum flag to check command - ([a0feb8b](https://code.ornl.gov/research-enablement/acorn/commit/a0feb8ba74564ea79e1dedee3cdeb3f72159e100))

### 🐛 Bug Fixes

- *(ci)* Add musl dependencies to CI setup - ([57dc6f7](https://code.ornl.gov/research-enablement/acorn/commit/57dc6f7b7a8d6a763d69ba7e5126dcaf62531e76))
- *(docs)* Use correct name to function link - ([ef00dca](https://code.ornl.gov/research-enablement/acorn/commit/ef00dca36d0af8a11edfcca211ebddb89dcde1fe))
- Move powerpoint extract function to util - ([a20388d](https://code.ornl.gov/research-enablement/acorn/commit/a20388db062df98631f1b1a93f2bc5121c34c0e8))

### 📚 Documentation

- Add explanatory note to extract_zip doc comment - ([e91168f](https://code.ornl.gov/research-enablement/acorn/commit/e91168f46bbb0ea3e785cf640bd9c17f480e4bc2))
- Improve links section of acorn-py pypi configuration - ([8de2df1](https://code.ornl.gov/research-enablement/acorn/commit/8de2df141fbec5b9883bce3d22813c015e08c403))
- Build CHANGELOG after new version - ([f897279](https://code.ornl.gov/research-enablement/acorn/commit/f897279f476c90d831c34e556463a1bcf70dfbc1))

### ⚙️ Miscellaneous Tasks

- Parameterize config on binary name - ([81c0704](https://code.ornl.gov/research-enablement/acorn/commit/81c070418588b2bf5aa57ef5f108f59b43926640))

### 🚧 In Progress

- Bump version to test musl build - ([cb1c3f8](https://code.ornl.gov/research-enablement/acorn/commit/cb1c3f831b1fed4a487c626abac83790a49cf74c))
## [0.1.31](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.30..v0.1.31) - 2025-10-19

### 🚀 Features

- Add format_orcid to python API - ([6af8345](https://code.ornl.gov/research-enablement/acorn/commit/6af8345c8edda704bbf24999f0961c97f06698be))
- Make --merge-request flag get files from GitLab API for more consistent behavior - ([7bfe457](https://code.ornl.gov/research-enablement/acorn/commit/7bfe457fd6eb81c2d7c97f7b5dc933391c23be0e))
- Augment ProgrammingLanguages enum and move it to schema module - ([a7b6760](https://code.ornl.gov/research-enablement/acorn/commit/a7b6760bda8eac99b3a75b6c81e55dcf8d36d6b5))
- Add methods to Location and moar docs - ([e298be4](https://code.ornl.gov/research-enablement/acorn/commit/e298be425b1c7712e7abb051e0bd47c1ea12903a))
- Improve ORCID support and fix some issues - ([f5fd13b](https://code.ornl.gov/research-enablement/acorn/commit/f5fd13b8109c8548dcfeead4452d035cda24262d))
- Improve default ORCID behavior and add some docs - ([d0b1e11](https://code.ornl.gov/research-enablement/acorn/commit/d0b1e117c8132889f8b04a044adca2994fae1446))
- Build out PID module and improve ORCID validation - ([5451201](https://code.ornl.gov/research-enablement/acorn/commit/5451201b977ec41cd09479f113d8e8a4873bc5c6))
- Use is_doi str method in validation - ([e1015a7](https://code.ornl.gov/research-enablement/acorn/commit/e1015a7c1764dfd05c2ad85a17c94a7d8b86ca09))
- Improve persistent identifier user experience with traits - ([230354f](https://code.ornl.gov/research-enablement/acorn/commit/230354f1f033fd38a5c5acb4db978103811b3258))
- Create PID module - ([4859eb4](https://code.ornl.gov/research-enablement/acorn/commit/4859eb4308312b5dcc39497358d1776e56b3a875))
- Separate doctests from tests; add module docs to util module - ([cdf47d8](https://code.ornl.gov/research-enablement/acorn/commit/cdf47d82939b3e8a4d22922b05fec90622e4aabc))
- Add docs to util and create to_absolute_string impl and function (rename of path_to_string) - ([ee2acb1](https://code.ornl.gov/research-enablement/acorn/commit/ee2acb1e13c20675d05f682e6e7148b5fff4d3d8))
- Add format_doi to acorn python API - ([b848264](https://code.ornl.gov/research-enablement/acorn/commit/b8482641999e2598480598738d07246f902b5b2b))
- Initial input and output stream handling capability - ([02a0360](https://code.ornl.gov/research-enablement/acorn/commit/02a0360322716da5ad05f5fb1042a492644f73ab))
- Initial DOI parsing and formatting capability - ([6ffbb21](https://code.ornl.gov/research-enablement/acorn/commit/6ffbb21f342b7bf9b6e44c2698853428c79528ac))
- Make checksum return an option - ([0db3400](https://code.ornl.gov/research-enablement/acorn/commit/0db34005490b156ee85e898f653af7ae44407185))
- Add ROR values to sponsors CSV - ([0bd90a0](https://code.ornl.gov/research-enablement/acorn/commit/0bd90a071b2240ad8dd0d1bc278c8f8590e20fca))
- Add snake_case function - ([f211f5f](https://code.ornl.gov/research-enablement/acorn/commit/f211f5f869511bbf91b0c4e789d011c8c66946a8))
- Replace titlecase crate with convert-case crate - ([670e1b3](https://code.ornl.gov/research-enablement/acorn/commit/670e1b32e83690626e09cf8a19d1e1ad4fb8d2bf))
- Update Label impl to include fmt for each method and methods used in more contexts - ([94e0625](https://code.ornl.gov/research-enablement/acorn/commit/94e06254a6919574ca411b2ac54d45d7aa28f208))
- Add nanoscale keyword - ([5fec849](https://code.ornl.gov/research-enablement/acorn/commit/5fec84987de16636b405179daadbad1bf502fb17))
- Add more partners from IMPACT and add ROR values to every partner - ([fde3184](https://code.ornl.gov/research-enablement/acorn/commit/fde31847b1647fedf6fa2e4480f4d050b79f153d))
- Add partners listed in IMPACT - ([6793ca6](https://code.ornl.gov/research-enablement/acorn/commit/6793ca66cff2a4bd454f76c570786984fd01a893))
- Add ResearchActivity class to acorn-py - ([224f97b](https://code.ornl.gov/research-enablement/acorn/commit/224f97b26af31e34e0ffb572e8a6c24a9912ed9d))
- Add more validate functions to acorn-py; add linting to python project - ([3823b8f](https://code.ornl.gov/research-enablement/acorn/commit/3823b8fd54f89767233d32701c617bce7a49b074))
- Add more validate functions to acorn-py - ([3aa00e2](https://code.ornl.gov/research-enablement/acorn/commit/3aa00e2d954b98d4110563b4b73fd0462b64939b))
- Make validate submodule of schema submodule in acorn-py - ([83a0c93](https://code.ornl.gov/research-enablement/acorn/commit/83a0c931e8791bd9a36f2d7dadc4da668255ba7e))
- Add validaate submodule to acorn-py - ([90450c1](https://code.ornl.gov/research-enablement/acorn/commit/90450c154cb4e99c5e9d5c7501071f038ae9e296))
- Add is_orcid to acorn-py - ([09769cb](https://code.ornl.gov/research-enablement/acorn/commit/09769cb25f46d391cb53ab3145629a84c72fa216))
- Configure acorn-py for proper development using pixi - ([356b7c2](https://code.ornl.gov/research-enablement/acorn/commit/356b7c2a1c8920610dc29d110ba5efc571ef5cd7))
- Add maturin develop make task - ([f817254](https://code.ornl.gov/research-enablement/acorn/commit/f817254ab7c1461f8ab1905ad6e6515dc6bda069))
- Add ORCiD identifier to ContactPoint in RAD schema - ([36b330a](https://code.ornl.gov/research-enablement/acorn/commit/36b330a243e35f151390b0de6e9b1ed189544cca))
- Add initial project structure for Python package - ([0563c25](https://code.ornl.gov/research-enablement/acorn/commit/0563c25758826e8a53c9bb7e5931065903ce5ae1))
- Add validation to Unix epoch timestamp values in RAiD metadata - ([f31844a](https://code.ornl.gov/research-enablement/acorn/commit/f31844a10e779daa45df4c16192093657f5a5d58))
- Improve validation and refine RAiD schema based off using ARDC demo server - ([0b09b77](https://code.ornl.gov/research-enablement/acorn/commit/0b09b779e946c1fc34207edef8b41b6414373825))
- Integrate schema validation for RAiD metadata - ([a53fa70](https://code.ornl.gov/research-enablement/acorn/commit/a53fa707e3aaebcf303bb7879e7e9c56b83a2319))
- Make more RAiD field optional to parse second example RAiD response - ([36f8acb](https://code.ornl.gov/research-enablement/acorn/commit/36f8acb18a489d3598d482ccf70d25393a56716e))
- Initial RAiD parsing capability with test - ([ade6a3b](https://code.ornl.gov/research-enablement/acorn/commit/ade6a3bd8a732c2321be30bac84d38d785db58b6))
- Add offline support to data structures and initial test data - ([20a51fe](https://code.ornl.gov/research-enablement/acorn/commit/20a51fef992d554c138c6efc4c2405697dc0525f))
- More Location integration - ([aa62488](https://code.ornl.gov/research-enablement/acorn/commit/aa62488d9ed85936b7592da10cb583fef76b107d))
- New initial implementation of Location with more explicit design - ([5e5ea56](https://code.ornl.gov/research-enablement/acorn/commit/5e5ea56a27b5f55a212b265795e7cbb1d349e325))
- Initial implementation of Location enum for repository URIs - ([dcd7063](https://code.ornl.gov/research-enablement/acorn/commit/dcd7063c5d840a710b78e660852820d349b7d1b4))
- Add polars and initial dataframe creation capability - ([72f7f33](https://code.ornl.gov/research-enablement/acorn/commit/72f7f33ea866981e13d06bf9ff670c592f728159))
- Unify handling of check results and set the stage for CSV output - ([1542b11](https://code.ornl.gov/research-enablement/acorn/commit/1542b116747c5010a0b5fb6b7c2d808e2b8d4e3b))
- Standardize readability checks under Check struct - ([7c2131f](https://code.ornl.gov/research-enablement/acorn/commit/7c2131fdae792c8c457544f81dbc8098db52e9e4))
- Standardize prose analysis checks under Check struct - ([f236332](https://code.ornl.gov/research-enablement/acorn/commit/f236332904928ac632e9d948c41101d400e1dae7))
- Update readability constants to map to 12th grade reading level - ([18ac764](https://code.ornl.gov/research-enablement/acorn/commit/18ac764e974b06a827004956283ecb063c1ad49c))
- Consolidate check structs into a single struct, Check - ([ed75866](https://code.ornl.gov/research-enablement/acorn/commit/ed75866bf613a1944debf377afed304011aa6847))

### 🐛 Bug Fixes

- *(ci)* Execute commands in same context within make task - ([38163fc](https://code.ornl.gov/research-enablement/acorn/commit/38163fc97852c8accf9ab1c617de4050081c7065))
- *(ci)* Restore script in releases job - ([8ed80ef](https://code.ornl.gov/research-enablement/acorn/commit/8ed80efc28fbc08d952721cb389641f1e10f1545))
- *(ci)* Restore publish-release job name - ([9b8ee4c](https://code.ornl.gov/research-enablement/acorn/commit/9b8ee4c9b4ea5896ef2a9e3fec264b17fba50339))
- *(docs)* Resolve lint issues in docs - ([ae7ab16](https://code.ornl.gov/research-enablement/acorn/commit/ae7ab1663512ba8fd082738dd2705d78f7138af8))
- *(tests)* Make new util doctests pass - ([338f842](https://code.ornl.gov/research-enablement/acorn/commit/338f8423afc101c3a45aef1fbcbdbb093e549850))
- *(tests)* Update doi python tests to allow schema uri - ([524f271](https://code.ornl.gov/research-enablement/acorn/commit/524f271a8b2b9c087708161a40377630c0f23db1))
- *(tests)* Use DOI with invalid schema uri - ([81adef4](https://code.ornl.gov/research-enablement/acorn/commit/81adef41b301b65d77b45de5f1cbda1a0f22c9c9))
- Import DOI and then use is_valid in doctest - ([46233f3](https://code.ornl.gov/research-enablement/acorn/commit/46233f3a1e4a9411b79399ea1ccc25ca0864f7ca))
- Change how readability variables are read from env file; update tarpaulin to support py03 - ([f04f37f](https://code.ornl.gov/research-enablement/acorn/commit/f04f37ff80a499eb5aebcb1350d54579ff1cacf5))
- Remove deserialize attribute from contactPoint.identifier; Improve read error messages - ([e626a1d](https://code.ornl.gov/research-enablement/acorn/commit/e626a1d189fe79fa7853403e45f0d13e3c863062))
- Use unique name for python export - ([b654d5d](https://code.ornl.gov/research-enablement/acorn/commit/b654d5dda161604df82ba6937bea650383e51a07))
- Add nested validation to RAiD metadata - ([cf3fecf](https://code.ornl.gov/research-enablement/acorn/commit/cf3fecfcf5124cf9833658c5441956a778a9cff8))
- Added CESER - ([3ed033b](https://code.ornl.gov/research-enablement/acorn/commit/3ed033b884def1903eea00f1028cf923e2e95214))

### 📚 Documentation

- Improve docs with more examples, links, and words - ([ec82f0a](https://code.ornl.gov/research-enablement/acorn/commit/ec82f0abdda451a4f7082127dcc27bb9867325f1))
- Add tested cases to SemanticVersion docs - ([b98248e](https://code.ornl.gov/research-enablement/acorn/commit/b98248ed8af8ef41e1d23a9143496468b8cba3fb))
- Add ACORN overview to README - ([0a91e7c](https://code.ornl.gov/research-enablement/acorn/commit/0a91e7ca5c0f3e1206a024a909556c2acb70d9f9))
- Improve acorn-py documentation and configure acorn-py lib name to avoid internal conflicts - ([81a253e](https://code.ornl.gov/research-enablement/acorn/commit/81a253e5258b10064f57095f0c92e609d59b8f70))
- Improve acorn-py documentation - ([71037ac](https://code.ornl.gov/research-enablement/acorn/commit/71037ac52c3246d70cfc450e94cf90c67da09f58))
- Add acorn-py to design docs - ([6cc4f20](https://code.ornl.gov/research-enablement/acorn/commit/6cc4f20864de3bdb8e92d8aa219e24cb0763c725))
- Clear doc lint issues - ([72424bf](https://code.ornl.gov/research-enablement/acorn/commit/72424bf722590dbe475f6c903b00997e4bdf64d7))
- Add install instructions for dev dependencies - ([5fef301](https://code.ornl.gov/research-enablement/acorn/commit/5fef30155f0c4960d4e36aa5b2726fa78be9b494))
- Add openssl setup to contributing docs - ([3ab573b](https://code.ornl.gov/research-enablement/acorn/commit/3ab573b4c97840ab99e15489fcdeb88975cd62a4))

### 🧪 Testing

- Add case that exercises checksum empty string return - ([41e5a4d](https://code.ornl.gov/research-enablement/acorn/commit/41e5a4d633aaab3c5abe5bba1edb9feb21a142a2))
- Build coverage report - ([caf823b](https://code.ornl.gov/research-enablement/acorn/commit/caf823b07794c45ca2deecc9ceb012ee44d54e64))

### ⚙️ Miscellaneous Tasks

- Add job for running python tests - ([a1697b1](https://code.ornl.gov/research-enablement/acorn/commit/a1697b1b64dedefa8f613272116ba2986eb274fd))
- Turn on CI debug services - ([cb459c3](https://code.ornl.gov/research-enablement/acorn/commit/cb459c3e4be6492153c9cbfa8ca4e48e10a3a3ce))
- Update changelog and refine CI config - ([7a87921](https://code.ornl.gov/research-enablement/acorn/commit/7a879218b6f79041e2701f767baf767dc08e6ed4))
- Move documentation publishing to make task - ([0db5de3](https://code.ornl.gov/research-enablement/acorn/commit/0db5de39094040901090e0c43e8458b97d2b3521))

### 🚧 In Progress

- Working on DOI struct and DOI parsing - ([e709421](https://code.ornl.gov/research-enablement/acorn/commit/e7094213e1fa07e0cf38ad6bf3f50d905edeb5b4))
- Initial addition of ResearchActivity to python API - ([3128771](https://code.ornl.gov/research-enablement/acorn/commit/31287711998a2b5ac83bdc8399b77e5f9e4c1632))
- Start adding support for local repository URIs - ([6dc296e](https://code.ornl.gov/research-enablement/acorn/commit/6dc296e180c9f002a78af2a506f61244758c8ff4))

### 🤓 Developer Experience

- Add Peacock configuration to project - ([46451d5](https://code.ornl.gov/research-enablement/acorn/commit/46451d50ad9963849c45d736963822d32416f2b2))
- Add setup make task for install cargo dev dependencies - ([66bfdeb](https://code.ornl.gov/research-enablement/acorn/commit/66bfdebb7d98dcaf197d5a903ac15471079e5420))
- Move acorn-py tasks into acorn-py directory (makefile and pyproject) - ([96513c4](https://code.ornl.gov/research-enablement/acorn/commit/96513c4cad356228a9afa3fcf6157cf61a882ea6))
- Refactor makefile to use task list for PHONY declarations - ([bd35437](https://code.ornl.gov/research-enablement/acorn/commit/bd3543793763c3c93e1e12a38d2cace8dd844a17))
- Stop publishing source distribution to pypi - ([c624e62](https://code.ornl.gov/research-enablement/acorn/commit/c624e62f8479997069351b23d24c5d691d3d96ee))
- Add make task for publishing PyPI package - ([14ea510](https://code.ornl.gov/research-enablement/acorn/commit/14ea5105c55b304bf77c849f59d6e1df955c3990))
- Refine pixi lint task definition - ([169c2b0](https://code.ornl.gov/research-enablement/acorn/commit/169c2b00cb289d8ed50ead31b34f81a3f68b20c3))
- Create scripts directory and move scripts into it - ([7363e70](https://code.ornl.gov/research-enablement/acorn/commit/7363e70edfcf800f7096bf57889a797fe04762cf))
- Perform git pull with tags before generating changelog with git-cliff in makefile - ([2a05670](https://code.ornl.gov/research-enablement/acorn/commit/2a05670143e6fbdf703ccfb1dd1dcfe66e4c0b12))
- Add openssl setup script for windows - ([919ba66](https://code.ornl.gov/research-enablement/acorn/commit/919ba6632bae83a05957e5dee174c4df43a418fe))
## [0.1.30](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.29..v0.1.30) - 2025-09-12

### 🚀 Features

- Add info output to export command - ([c4df2a0](https://code.ornl.gov/research-enablement/acorn/commit/c4df2a03a269391b5f3c47839a8c99ce2d5011ae))
- Abstract ignoring files across every file finding interface - ([90088f8](https://code.ornl.gov/research-enablement/acorn/commit/90088f8fc1bb9992f87ccc309d806fc892d6fa95))
## [0.1.29](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.27..v0.1.29) - 2025-09-12

### 🚀 Features

- Make meta.ror a vector - ([440a146](https://code.ornl.gov/research-enablement/acorn/commit/440a14634df30157c017f16b2bc781d674e9d39d))
- Add WARP context file (also really good docs) - ([956bca1](https://code.ornl.gov/research-enablement/acorn/commit/956bca1bd679c6746214d8031bc75baa4baf2450))
- Add toml to MimeType - ([9150f53](https://code.ornl.gov/research-enablement/acorn/commit/9150f53f4fbd1dfae79c7ae86daa2d6503b1382d))

### 🐛 Bug Fixes

- *(ci)* Fetch main branch to enable git related tests - ([cf75cdf](https://code.ornl.gov/research-enablement/acorn/commit/cf75cdfffca50bc1f09f61e11a55a3a6a9ebfc0c))
- Make  extension lowercase for git filter comparison to fix check --commit - ([dd38315](https://code.ornl.gov/research-enablement/acorn/commit/dd38315eed6a48d18d06537820cc5a66fe076045))
- Install Node v22 on bucket image - ([bd51699](https://code.ornl.gov/research-enablement/acorn/commit/bd51699eeece1a5766a9b136b3e9d28a2014e612))

### 📚 Documentation

- Add project files for documentation - ([65f1f45](https://code.ornl.gov/research-enablement/acorn/commit/65f1f45be6ba093c5b8fd2e10fbca58decf65ccd))

### 🧪 Testing

- Add CFF mimetype and tests for git file functions - ([c6f8a8b](https://code.ornl.gov/research-enablement/acorn/commit/c6f8a8bda9f3023bf2e192b75a4ce26578e65c4a))
## [0.1.27](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.26..v0.1.27) - 2025-09-08

### 🚀 Features

- Add release profile options - ([35fff36](https://code.ornl.gov/research-enablement/acorn/commit/35fff36788e60d9e1566bdebb1db28dcb084cac6))

### 🐛 Bug Fixes

- *(docs)* Update cargo install instructions - ([7e3b9f8](https://code.ornl.gov/research-enablement/acorn/commit/7e3b9f8dcf66a708d5898280a4031e9f6d8867b1))
- Add markdown mime type and fix git commit filtering issue - ([7c2cdb7](https://code.ornl.gov/research-enablement/acorn/commit/7c2cdb77f866e9e4b36803a9558270afb5e1120d))
- Remove unused poster assets - ([784e203](https://code.ornl.gov/research-enablement/acorn/commit/784e20388f1b6eacbb45b6129512d2d12b0e6bd3))

### 📚 Documentation

- Pull tags and rebuild CHANGELOG - ([5952bf7](https://code.ornl.gov/research-enablement/acorn/commit/5952bf7f6fae835a8b830453207ebef7889ddb0e))
- Update CLI to use acorn-lib crate; separate crate name and binary name - ([24c5d65](https://code.ornl.gov/research-enablement/acorn/commit/24c5d65aedbd16c43435c4e6425025b6bea20c33))
- Update CHANGELOG - ([1c2de91](https://code.ornl.gov/research-enablement/acorn/commit/1c2de91b38ca9f3d1ce15396a3a9dbd895cd7205))
## [0.1.26](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.25..v0.1.26) - 2025-09-07

### 🚀 Features

- Fix thread limiting and parent function - ([7e87da2](https://code.ornl.gov/research-enablement/acorn/commit/7e87da27b936be1c95113445f0e2132fc91654d9))
- Add thread limit option - ([406f94e](https://code.ornl.gov/research-enablement/acorn/commit/406f94e7cce555f177a50004dda7c71b41ab2780))
- Clean up acorn tasks; clear security some issues in acorn dockerfile; point to acorn binary - ([12db8da](https://code.ornl.gov/research-enablement/acorn/commit/12db8dae9acc12b74cbd52a16e00186201f18726))

### 🐛 Bug Fixes

- Base bucket image on ubuntu; refine tagging tasks - ([69d09f4](https://code.ornl.gov/research-enablement/acorn/commit/69d09f43538db18ca59ff1fee42a67f36171b7f6))

### 📚 Documentation

- Update lib README for publishing - ([a1190f7](https://code.ornl.gov/research-enablement/acorn/commit/a1190f794185760bbc52889422ecc2a8f6c2e349))

### 🤓 Developer Experience

- Refine bucket image build tasks and add platform option - ([a382b13](https://code.ornl.gov/research-enablement/acorn/commit/a382b139489c8226fae089aca2ddd19d23225c38))
- Add docker make tasks - ([1be9ca7](https://code.ornl.gov/research-enablement/acorn/commit/1be9ca7d1aa61d09c40a209595508ad13e1d9f6f))
- Update docker image make tasks - ([21c6742](https://code.ornl.gov/research-enablement/acorn/commit/21c6742753dccc22cc9425d02df8f63773436edd))
## [0.1.25](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.24..v0.1.25) - 2025-09-05

### 🚀 Features

- Refactor schema validation checks to support collected output - ([aab6309](https://code.ornl.gov/research-enablement/acorn/commit/aab63093baf6f928f4714a2679de920ccf4c7cc0))
- Configure readability metric from CLI and dotenv file - ([ee58c0d](https://code.ornl.gov/research-enablement/acorn/commit/ee58c0d9ddbe0a9e521287833e5cc419162f7866))
- Refine website checking capability using LinkCheck struct - ([ae61400](https://code.ornl.gov/research-enablement/acorn/commit/ae61400cfdd079c06c2563c6ffe16d3c714a6bdc))
- Initial capability to check meta.websites links for accessibility - ([315c66d](https://code.ornl.gov/research-enablement/acorn/commit/315c66d0bf0095d573082d51000e3a3062f1183d))
- Initial capability to configure readability checks with dotenv file - ([11fad82](https://code.ornl.gov/research-enablement/acorn/commit/11fad821c8dd9b995ce0a2e0b3c29b5dcc3e56ec))
- Initial readability check subcommand capability - ([d061e58](https://code.ornl.gov/research-enablement/acorn/commit/d061e58f00324fa8076abed3529735ab53a65b67))
- Initial capability to create citation from DOI value - ([667a757](https://code.ornl.gov/research-enablement/acorn/commit/667a75704ded4f59b6c066f41c62207972924e61))
- Improve DOI validation - ([ec943ff](https://code.ornl.gov/research-enablement/acorn/commit/ec943ffb9203f3597527b51026add228534a71c4))
- Initial data modeling for CiteAs API communication - ([9284f3c](https://code.ornl.gov/research-enablement/acorn/commit/9284f3cf7d8e28563d9a6fa1f792ccb90c5ab985))
- Add rad and raid subcommands to schema command - ([394953a](https://code.ornl.gov/research-enablement/acorn/commit/394953a1c9f0a876c4ab48b238183ff785fce5c8))
- Add technology and keywords from review of ornl software catalog and linux foundation - ([ab8862e](https://code.ornl.gov/research-enablement/acorn/commit/ab8862e506b2e65dda5c29a6509fdbcfe853d36d))
- Add technologies and URLs for technology values - ([eed52a7](https://code.ornl.gov/research-enablement/acorn/commit/eed52a76df0908358f029b8de0a28ca9e44c7b76))
- Add multi-extension support to git files functions - ([7477a09](https://code.ornl.gov/research-enablement/acorn/commit/7477a09909b15b9a3d3e08df6ffaeaff5608fd21))
- Add multi-extension support to files_all function - ([3487018](https://code.ornl.gov/research-enablement/acorn/commit/3487018059ba08246d841c4de4479dfc04239125))
- Improve MimeType code - ([83ebe99](https://code.ornl.gov/research-enablement/acorn/commit/83ebe990c3db2962c8e0d22d3d37e2bc0211b984))
- Replace Extension with MimeType and remove Extension - ([630182d](https://code.ornl.gov/research-enablement/acorn/commit/630182d52ba99259992620e0d0cd42e1a3a60c1f))
- Add itertools to lib and make files_all return unique paths - ([d954e9d](https://code.ornl.gov/research-enablement/acorn/commit/d954e9dd64eab19cc4c51a0b7d340b294aaee4ff))
- Add support for format command to return YAML - ([c885ee7](https://code.ornl.gov/research-enablement/acorn/commit/c885ee76e724a413f950d2a2d4b4e26c26237a7f))
- Add initial support for YAML format RAD - ([4e7aa3d](https://code.ornl.gov/research-enablement/acorn/commit/4e7aa3d93494d511bb8289c90be5453d79f861b3))
- Add support for YAML format buckets configuration files - ([8f62c4b](https://code.ornl.gov/research-enablement/acorn/commit/8f62c4bc959461f50eb2817c26d650cc2e52d8e3))
- Add full support for GitHub buckets - ([7365c73](https://code.ornl.gov/research-enablement/acorn/commit/7365c73852d0cd3e125c533b3f0f8dfc72db3e0f))
- Improve GitHub bucket support up to pagination - ([c4402bd](https://code.ornl.gov/research-enablement/acorn/commit/c4402bd45c1f9a510dcfa958de82397ce3ef0508))
- Initial GitHub viability - ([0353c72](https://code.ornl.gov/research-enablement/acorn/commit/0353c72288a407fb963e29f2261c7d2aabac8c89))
- Improve GitHub bucket support - ([3f60648](https://code.ornl.gov/research-enablement/acorn/commit/3f606489de28ccdae4a87bb69c92591024117bad))
- Add more validation methods; use raid module within schema - ([3a0a6ef](https://code.ornl.gov/research-enablement/acorn/commit/3a0a6ef456845b794b28e040f4727f652383f1f7))
- Make meta.doi a vector and re-order meta attributes - ([90c3696](https://code.ornl.gov/research-enablement/acorn/commit/90c369641188e1c8a87a70dee9b1ceb20d4c8b9f))
- Finish RAiD metadata schema module and refine docs - ([5220e35](https://code.ornl.gov/research-enablement/acorn/commit/5220e3598ebc0684aed32d1cb607e98774057db4))
- More raid metadata schema development - ([abbd956](https://code.ornl.gov/research-enablement/acorn/commit/abbd956391364d5de4fdf216055847ae45e5d692))
- More improvements to raid metadata structure - ([4b0593d](https://code.ornl.gov/research-enablement/acorn/commit/4b0593d0bd75968a2850f02ae3daa99034bfec8f))
- Initial addition of raid metadata schema data modelling - ([4010ebb](https://code.ornl.gov/research-enablement/acorn/commit/4010ebb5765d606a82a9f871a08f4b53a4743bcb))
- Initial creation of raid metadata schema data module - ([e038da2](https://code.ornl.gov/research-enablement/acorn/commit/e038da28c36c7a234a53be43120e01dcc1426f5d))
- Initial creation of link command API - ([a79199a](https://code.ornl.gov/research-enablement/acorn/commit/a79199a368cc1c578d605d7e4cd1b0f466389fe0))
- Improve format command processing - ([801131c](https://code.ornl.gov/research-enablement/acorn/commit/801131c29b763e43010656bee7821fcf52c1d185))
- Add consortium to organization types - ([6cf02df](https://code.ornl.gov/research-enablement/acorn/commit/6cf02dfa3ab55f25fd6e4422b67079a1e6088fde))
- Add statistics to keywords - ([0bf752c](https://code.ornl.gov/research-enablement/acorn/commit/0bf752c8bd985bb0d414fe5361223fea23478f6e))
- Add computer-vision keyword; update name of ISED organization - ([863cb56](https://code.ornl.gov/research-enablement/acorn/commit/863cb56697bce4c7b2b8d5d9e60a234b00337f7c))
- Rename acorn-cli package to acorn in toml manifest - ([454ce18](https://code.ornl.gov/research-enablement/acorn/commit/454ce18effcd21080a47769188600b8a82cd7109))
- Improve support for working with Gitlab and GitHub repositories - ([57cbb8d](https://code.ornl.gov/research-enablement/acorn/commit/57cbb8d83f5952d273b4bd5b47cad03f4530318b))
- Add ability to get latest release from a repository - ([32bdb13](https://code.ornl.gov/research-enablement/acorn/commit/32bdb13ec46dfbfc480f8610116f37bb26712ba3))
- Offload styles to custom ORNL vale package - ([6560bbd](https://code.ornl.gov/research-enablement/acorn/commit/6560bbd3023944e4993e5b5830bf434a6678e3da))

### 🐛 Bug Fixes

- *(docs)* Use correct punctuation for eg - ([7574b26](https://code.ornl.gov/research-enablement/acorn/commit/7574b26987b4daff0039aa5bf2f0c83d4f7318c4))
- *(tests)* Skip non-functional doctest - ([f5568ef](https://code.ornl.gov/research-enablement/acorn/commit/f5568ef5d98611088a5f87d6b3eed08577ff9483))
- Make GitLab releases code useful for any GitLab instance - ([ff60390](https://code.ornl.gov/research-enablement/acorn/commit/ff6039074e1fd9bc21b3a06c092fe1b5213c3d69))
- Update build make task - ([016d92a](https://code.ornl.gov/research-enablement/acorn/commit/016d92a3c05ad6a199b9f850b3a65a479906c7f4))
- Make format resolve parent directory when file name in cwd is passed - ([aa1ce2d](https://code.ornl.gov/research-enablement/acorn/commit/aa1ce2d793054f956928652840ff636b5cf30b0d))
- Update bucket test and replace println with trace - ([8021392](https://code.ornl.gov/research-enablement/acorn/commit/8021392eb3b8fc86573c8602dcf5e4891e7ba8bf))
- Formatting updates to paper.md - ([b6c32f5](https://code.ornl.gov/research-enablement/acorn/commit/b6c32f5068460955327f3146f09346afc9b72b8f))
- Resolve organizations with no alternative name - ([86d494e](https://code.ornl.gov/research-enablement/acorn/commit/86d494efd17d59775ec3038bc17d7e7c3f2e5a57))

### 📚 Documentation

- Improve readme install instructions - ([6eab335](https://code.ornl.gov/research-enablement/acorn/commit/6eab33585a0e6e9a0670c76c40abc744eb9d6efc))
- Update research areas examples - ([f69c149](https://code.ornl.gov/research-enablement/acorn/commit/f69c149dbb388358a82b50f1ea8b49a2d5f61c23))
- Update sections documentation (with examples) - ([a402c11](https://code.ornl.gov/research-enablement/acorn/commit/a402c11a73db20b62b93788fe62b1275605bff6e))
- Add acorn emoji - ([2c32fff](https://code.ornl.gov/research-enablement/acorn/commit/2c32fff14012416a3c1128e64b86998b1b4fba8f))
- Standardize schema module documentation with raid docs - ([a59db02](https://code.ornl.gov/research-enablement/acorn/commit/a59db02e8182cb04e26f2cb2cd1d830cb9da518b))
- Update module documentation - ([dd54aff](https://code.ornl.gov/research-enablement/acorn/commit/dd54aff3f758faa052478a5d648e8a489d32bac6))

### 🧪 Testing

- Skip Github repository release test because sporadic failures - ([4b3c1b4](https://code.ornl.gov/research-enablement/acorn/commit/4b3c1b49dfdf362b811f17244133d9e980ab2083))
- Add invalid case to DOI tests - ([2db9570](https://code.ornl.gov/research-enablement/acorn/commit/2db9570f0c538c3ab7ead19360f5d3017bdb8791))
- Add tests for niche use cases of util::parent - ([ba1db11](https://code.ornl.gov/research-enablement/acorn/commit/ba1db1112e6bad634e78a6f666198e12ddc79801))

### Feat

- Initial markdown draft file for JOSS paper - ([1cdacae](https://code.ornl.gov/research-enablement/acorn/commit/1cdacaed5fee77c5a9925efe37eb19176271fc7e))
## [0.1.24](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.23..v0.1.24) - 2025-07-22

### 🚀 Features

- Update logo images and center image and caption for PDF fact sheet output - ([2fc0e68](https://code.ornl.gov/research-enablement/acorn/commit/2fc0e6807b36dd40819bb36c48d0c77462ca5426))
- Add todo for downloading files from bucket using project path instead of id - ([63a8558](https://code.ornl.gov/research-enablement/acorn/commit/63a85589e57c2fdd1d0535d434d1051486f85348))
- Add audit make task and remove clap-help dependency to improve security posture - ([5e4b987](https://code.ornl.gov/research-enablement/acorn/commit/5e4b98713c60b435c501c2166b0224c9ccc6d963))
## [0.1.23](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.22..v0.1.23) - 2025-07-18

### 🐛 Bug Fixes

- Make path positional argument optional - ([2b4fb86](https://code.ornl.gov/research-enablement/acorn/commit/2b4fb868fabe4a48c70a50b5ddb727de7d7d22df))
## [0.1.22](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.21..v0.1.22) - 2025-07-07

### 🚀 Features

- Improve flexibility of util module with generic types - ([7b2f246](https://code.ornl.gov/research-enablement/acorn/commit/7b2f24670b34a5fea0cb16c4dc145346e2a54d7a))
- Improve flexibility with generic types for most iterators and PathBufs - ([d020701](https://code.ornl.gov/research-enablement/acorn/commit/d020701b41bfcd14dc1ea26fea297a02e7e65b03))
- Improve predictability by creating cli feature and Convert trait - ([4320169](https://code.ornl.gov/research-enablement/acorn/commit/4320169de919976e49c8b282ded6abb2928b29c6))
- Implement common traits on ResearchActivity to improve interoperability (C-COMMON-TRAITS convention) - ([ad80264](https://code.ornl.gov/research-enablement/acorn/commit/ad802649bb24d873cbd993925062c70e43521226))
- Remove all get_ prefix from functions in acorn-cli (C-GETTER convention) - ([2961d7b](https://code.ornl.gov/research-enablement/acorn/commit/2961d7b5699e11760db077cbafd0894c999bc579))
- Remove all get_ prefix from functions in acorn-lib (C-GETTER convention) - ([aa7bf9e](https://code.ornl.gov/research-enablement/acorn/commit/aa7bf9eb167c59988c5db0cadc85a99e731e0ca9))
- Remove get_ prefix from functions in lib util module (C-GETTER convention) - ([8ca0fc2](https://code.ornl.gov/research-enablement/acorn/commit/8ca0fc2414fb776e90a5ea75f5578a7c4762ab16))
- Update all dependencies - ([fc19749](https://code.ornl.gov/research-enablement/acorn/commit/fc19749da4ebdbbf1135228284e74deb5c91cdbc))
- Move export --schema to schema sub command - ([4f6a935](https://code.ornl.gov/research-enablement/acorn/commit/4f6a9351d763aa751208705e13b47439c7beb0a6))
- Update workspace dependencies - ([ce2c43a](https://code.ornl.gov/research-enablement/acorn/commit/ce2c43a4b784409b6a60dfa8c14b858b922bad2f))
- Refactor format instances with one variable to interpolate - ([a4b2b07](https://code.ornl.gov/research-enablement/acorn/commit/a4b2b078765339a48e26813d27f390eb4ff64cc9))
- Improve Rust API compliance by renaming some functions to remove get_ - ([d7cf5e8](https://code.ornl.gov/research-enablement/acorn/commit/d7cf5e85d7998a9df01e23513ffa448902c0985b))
- Convert --path to positional argument for better UX - ([f439891](https://code.ornl.gov/research-enablement/acorn/commit/f4398913a7601f8d8ba2c244637be16d63ae1140))
- Remove schema_type from Metadata - ([7191828](https://code.ornl.gov/research-enablement/acorn/commit/719182861c01ea4c0cdd1364c751da745dae5170))
- Add docs and validation - ([61f1fec](https://code.ornl.gov/research-enablement/acorn/commit/61f1fec8cbcef71fcceb1c2b70ba40a97983b5ae))
- Refactor Graphics -> Media - ([b900126](https://code.ornl.gov/research-enablement/acorn/commit/b90012625fa7e4db003d1f5b3bf0130febf8790c))
- Flatten sections struct to match updated schema - ([15ef489](https://code.ornl.gov/research-enablement/acorn/commit/15ef4895ebbea13edae8b2f564d27a0f0072b115))
- Create get_suffix util function - ([6d55f57](https://code.ornl.gov/research-enablement/acorn/commit/6d55f57da624faf6f78a05882b7484237735838b))
- Improve color and grammar of schema error output - ([8d7bacd](https://code.ornl.gov/research-enablement/acorn/commit/8d7bacd876ee0a50916a550bbb86129f6ffa8819))
- Make schema_type optional and deprecated ISO of schema refactor - ([9b76007](https://code.ornl.gov/research-enablement/acorn/commit/9b76007edfd0082e8be81cc16cc0795f734e4d41))
- Clear lint issues - ([bca0e0f](https://code.ornl.gov/research-enablement/acorn/commit/bca0e0f82d58d762d65c618e772511653f9f8078))
- Technical to approach; conflate approach across schemas - ([3b6558b](https://code.ornl.gov/research-enablement/acorn/commit/3b6558b9e8bbc9d754e348bfb14689f9287b28b1))
- Make achievement and capabilities optional; add to project sections - ([1c6ebb1](https://code.ornl.gov/research-enablement/acorn/commit/1c6ebb1a1691fd7335f7c0523b080243d46417b4))
- Replace outcomes with impact - ([b62ab83](https://code.ornl.gov/research-enablement/acorn/commit/b62ab83e70a65e19f7d8e7887ff6451a44d73523))
- Rename introduction to mission in project sections (include introduction as alias) - ([0a96554](https://code.ornl.gov/research-enablement/acorn/commit/0a965543e72272123dd7236e9616b4baa37c9679))
- Remove facilities from organization ISO of schema refactor - ([0084b3c](https://code.ornl.gov/research-enablement/acorn/commit/0084b3cdb73d48f393084e689ee61d22e70d5c74))
- Refine bucket download function - ([19db37f](https://code.ornl.gov/research-enablement/acorn/commit/19db37f5cb74a63214967e9dcc40c8d997a2b15b))
- Update docker image build make tasks - ([57195b6](https://code.ornl.gov/research-enablement/acorn/commit/57195b6c65393f124ab4817c9d4b7ce489cce316))

### 🐛 Bug Fixes

- *(docs)* Update link to acorn install instructions - ([18ecfdc](https://code.ornl.gov/research-enablement/acorn/commit/18ecfdc2b2601358dcf1f03c6daea91a7f38a671))
- *(docs)* Update ooxml module doc comments; allow missing_docs lint - ([bb2a259](https://code.ornl.gov/research-enablement/acorn/commit/bb2a259d2475c7ad939e8c3be4d13bfbe2748ef2))
- *(docs)* Remove license-file attribute in manifest files - ([45361dd](https://code.ornl.gov/research-enablement/acorn/commit/45361dd135a97864004ffcc03cbb1852fa69e9a8))
- *(tests)* Restrict some tests to linux - ([652f405](https://code.ornl.gov/research-enablement/acorn/commit/652f4058059fb7efadd94ac529604be82191b921))
- *(tests)* Update test fixtures to support removal of facilities - ([bebf052](https://code.ornl.gov/research-enablement/acorn/commit/bebf0527ee2ee665380aeade017800613af1552a))
- Update download command for new schema - ([5ea1098](https://code.ornl.gov/research-enablement/acorn/commit/5ea109872140e6ef3350dd1d41ab288ba9d2d599))
- Make serde throw error for unknown fields - ([5e2bb3d](https://code.ornl.gov/research-enablement/acorn/commit/5e2bb3d47dc1e26b5bb93fca316716ca83319056))

### 📚 Documentation

- Add doc comments for all remaining code; deny missing_docs lint rule - ([3c2014e](https://code.ornl.gov/research-enablement/acorn/commit/3c2014e933303a6bab57ec6bbd2bfffec8f01d8c))
- Add doc comments to schema, graph, and ooxml modules - ([e1fd160](https://code.ornl.gov/research-enablement/acorn/commit/e1fd160f239fb39c503a0376f219867acafa1e5c))
- Add more doc comments for modules; improve constants module documentation - ([d02e8dc](https://code.ornl.gov/research-enablement/acorn/commit/d02e8dc1fc81236c69e8d0f407a0cdbd9e4e598c))
- Add doc comments for validator module - ([5babbcc](https://code.ornl.gov/research-enablement/acorn/commit/5babbcc1aad001ae874fc33393e95f8d6c6cd21c))
- Add docs to mods and util files - ([8c95a36](https://code.ornl.gov/research-enablement/acorn/commit/8c95a36c5bd2f0a0b76fbeafea8c24bbe89d7b45))
- Add metadata to package manifests to improve documentation (C-METADATA convention) - ([d0f1a5f](https://code.ornl.gov/research-enablement/acorn/commit/d0f1a5fe2e10ef3c98710c300ce3906bd6accedd))
- Add seedling emoji to README - ([99f6efe](https://code.ornl.gov/research-enablement/acorn/commit/99f6efe8e2490756fc3fed7ebc1be1427f60e048))
- Add release badge to README - ([3db1a5d](https://code.ornl.gov/research-enablement/acorn/commit/3db1a5d5e6a94f886e3ee541ad9a8c6470c0b7ba))
- Add CHANGELOG generated using git-cliff - ([f0fda39](https://code.ornl.gov/research-enablement/acorn/commit/f0fda39327cb19fb5105a49b43837c9347db1c10))
- Improve help grouping and coloring; add minimal tests - ([18de14a](https://code.ornl.gov/research-enablement/acorn/commit/18de14a789e2fd0063e248bcec5b7edc784eb145))
- Add style to help output - ([996265b](https://code.ornl.gov/research-enablement/acorn/commit/996265b70b44aefab1ef30dd0502c915c0dde1e1))
- Add inline docs for meta.capabilities - ([a7157ff](https://code.ornl.gov/research-enablement/acorn/commit/a7157ffda136f357ce363a9d91b95db105dac556))
- Add docs for lib.rs - ([347f950](https://code.ornl.gov/research-enablement/acorn/commit/347f950c7489fafa94211ef68b01d19615c2cbdd))
- Add CITATION.cff and LICENSE content - ([6fd17e6](https://code.ornl.gov/research-enablement/acorn/commit/6fd17e675631130493117107ba14ed2b4fa1b1e4))

### 🚧 In Progress

- Initial addition of conventions check category - ([13611d8](https://code.ornl.gov/research-enablement/acorn/commit/13611d8a6525d0c47f14cfc5b39b1bac8a71c61d))
- Add structs for Media ISO replacing Graphic struct - ([95d47ee](https://code.ornl.gov/research-enablement/acorn/commit/95d47ee17382a29e30f993cd71262cd97333e47b))

### 🤓 Developer Experience

- Add make test for generating CHANGELOG - ([01f6f4b](https://code.ornl.gov/research-enablement/acorn/commit/01f6f4bfc84167b22900d4df968593ca2e614014))
## [0.1.21](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.20..v0.1.21) - 2025-06-06

### 🚀 Features

- Improve download error handling; remove highlights from download - ([21d9bf7](https://code.ornl.gov/research-enablement/acorn/commit/21d9bf714192fa68e0ae856a0f257b0b7462a344))
- Use minideb for docker image - ([acc6c4d](https://code.ornl.gov/research-enablement/acorn/commit/acc6c4d006bf61fad9d43ab07b66c92a61830ce9))
- Add docker ignore file - ([d457765](https://code.ornl.gov/research-enablement/acorn/commit/d45776538cdf1384cea4cdca369683d10c245ee2))
- Solve musl build; update acorn dockerfile - ([e27df82](https://code.ornl.gov/research-enablement/acorn/commit/e27df82b7852a33471df7d3668067007ce199999))
- Normalize phone number format during format command - ([14d60ae](https://code.ornl.gov/research-enablement/acorn/commit/14d60ae6515f3955bfd096db2033f4f6ad76ea6b))
- Add support for validation to detect fake phone numbers - ([ef8e22f](https://code.ornl.gov/research-enablement/acorn/commit/ef8e22f8e90dfd0a0990144bed3a0073cab169ae))
- Add --offline option to app as placeholder for future functionality - ([5f4259e](https://code.ornl.gov/research-enablement/acorn/commit/5f4259e0cf0b1605cf0b8431d8b203bd087a0654))
- V1 PPTX export capability - ([00a99af](https://code.ornl.gov/research-enablement/acorn/commit/00a99af7dd5d149bd060f8206494d50ae15f02db))
- Add national-security keyword - ([8ede80d](https://code.ornl.gov/research-enablement/acorn/commit/8ede80d5c21428ee7b152e8b09fd25a379a4861b))
- Add nano words to accept list - ([b9c8623](https://code.ornl.gov/research-enablement/acorn/commit/b9c8623be81907133740939d66fbdbdf8f887cea))
- Add OpenDB to technology.csv - ([d80e66f](https://code.ornl.gov/research-enablement/acorn/commit/d80e66f41acb494eaf757d52620c06c52c43f88e))
- Initial data modeling for OOXML paragraph object - ([3d3d1d9](https://code.ornl.gov/research-enablement/acorn/commit/3d3d1d947faa8ad0f13453d95c66f3cc8ba44d64))
- Add notes to pptx value interpolation - ([e517e34](https://code.ornl.gov/research-enablement/acorn/commit/e517e34cb16ad9d0c0d5cd9c3869d84cdc82a3b7))
- Add caption and subtitle to powerpoint value interpolation; update test reference pptx - ([bbedab3](https://code.ornl.gov/research-enablement/acorn/commit/bbedab36050079549d51aa1ce2b7ae32da1fea07))
- Add contact information to PPTX interpolation - ([c556c18](https://code.ornl.gov/research-enablement/acorn/commit/c556c189916724a2b46f4609fcbd6f1a80d5622a))
- Add Dockerfile for acorn - ([0689964](https://code.ornl.gov/research-enablement/acorn/commit/068996477b4ce586d77e3a645c5561b95019749a))
- Add database technologies to technology CV - ([20d4df8](https://code.ornl.gov/research-enablement/acorn/commit/20d4df8bf3e779ee30ded6495dbee9f7d89b45c4))

### 🐛 Bug Fixes

- *(tests)* Update highlight test fixtures to use new section schema - ([52f6897](https://code.ornl.gov/research-enablement/acorn/commit/52f68972f5b35b263b553e9cd8ec7863cdfa2017))
- Remove opendb from technology csv because it is internal - ([5c9e71d](https://code.ornl.gov/research-enablement/acorn/commit/5c9e71d0e1338bcc00a2d91151fc4da2d4c37b53))
- Added hyphen to NGA sponsors.csv - ([e8ed956](https://code.ornl.gov/research-enablement/acorn/commit/e8ed956162bb059777569fd9cbe04cda7762b575))
- Use branch instead of default branch for --merge-request option - ([57b174b](https://code.ornl.gov/research-enablement/acorn/commit/57b174b335b03c367d477cbe09c784da3faa6118))

### 📚 Documentation

- Add to vale sync error output - ([7288b18](https://code.ornl.gov/research-enablement/acorn/commit/7288b180e84b41569ba217430f50b40152a15b98))

### 🧪 Testing

- Prepare for working with XML and regex - ([e81b2af](https://code.ornl.gov/research-enablement/acorn/commit/e81b2af95585135c1cc01254a725db4cb74c66dc))
- Add highlight data for development and testing - ([ccfe209](https://code.ornl.gov/research-enablement/acorn/commit/ccfe2098511383b6a33cb621fce4704304ade49c))

### ⚙️ Miscellaneous Tasks

- Rename bucket dockerfile and create make task - ([7590d25](https://code.ornl.gov/research-enablement/acorn/commit/7590d25d6fde481f96ea4ed40da9f845b9353d17))
- Add Dockerfile for bucket CI - ([de6a361](https://code.ornl.gov/research-enablement/acorn/commit/de6a36101b134ad6f8134cccad1a87d72452a0d1))

### 🚧 In Progress

- Fix PPTX repair issue and interpolate technical approach - ([4313ea9](https://code.ornl.gov/research-enablement/acorn/commit/4313ea9e63bfc9706e3a55e39e106784ceb6d1f1))
- Initial interpolation of bullets ISO PPTX export - ([956446f](https://code.ornl.gov/research-enablement/acorn/commit/956446f6c3564977add1e7cbafe7941a9bad8325))
- Parse found paragraph element ISO PPTX export - ([0c4f394](https://code.ornl.gov/research-enablement/acorn/commit/0c4f3943620f5080df27ec0290669237f91c83c0))
- Initial capability to replace certain paragraph in XML - ([067b2a2](https://code.ornl.gov/research-enablement/acorn/commit/067b2a2efc6c104d51677e5b1dc75b35d735a582))
- Add more OOXML data structures ISO PPTX export - ([8197a79](https://code.ornl.gov/research-enablement/acorn/commit/8197a79e5604655e409111a38cc28d2311789aaa))
- Add replace function for interpolating values using mustache templates - ([a142f3c](https://code.ornl.gov/research-enablement/acorn/commit/a142f3cc4396cfc07a3605ac104ab6f88650400c))
- Initial interpolation capability for PPTX export - ([2fa763e](https://code.ornl.gov/research-enablement/acorn/commit/2fa763e97a75cb7db07faaefdfada54a84f3104b))
## [0.1.20](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.19..v0.1.20) - 2025-05-18

### 🚀 Features

- Add --merge-request and --branch options to check, format, and export sub-commands - ([bf781c5](https://code.ornl.gov/research-enablement/acorn/commit/bf781c568bba0d12b59178b9800bca574a830b39))
- Add git functions for future command options - ([24df5f3](https://code.ornl.gov/research-enablement/acorn/commit/24df5f309f395b612faf08aaa8c285fb14738099))

### 🐛 Bug Fixes

- *(tests)* Move git branch tests into separate function and ignore - ([660db6b](https://code.ornl.gov/research-enablement/acorn/commit/660db6bd4c3f0fd42187784bd9290944bb9b46df))
- *(tests)* Comment out git default branch test - ([98490d7](https://code.ornl.gov/research-enablement/acorn/commit/98490d7b24ea05c479d74939284c0c9415791504))
- Preprocess vale output to normalize paths on Windows - ([87dfdc6](https://code.ornl.gov/research-enablement/acorn/commit/87dfdc6748a3f2d320dcae15452fafe598cb73d9))
## [0.1.19](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.18..v0.1.19) - 2025-05-16

### 🚀 Features

- Add deepfake and basemap to prose accept list - ([b5c9671](https://code.ornl.gov/research-enablement/acorn/commit/b5c9671cdc4afafe90b859b1710221936a9fb019))
- Copy RAD image to PPTX export - ([cbe8ef6](https://code.ornl.gov/research-enablement/acorn/commit/cbe8ef62552687b11f3ad5fc05e5e8fe851f11ad))
- Add reference option for exporting to powerpoint - ([b538b54](https://code.ornl.gov/research-enablement/acorn/commit/b538b5466555a2baf6c6e727166607b688d6a42c))
## [0.1.18](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.17..v0.1.18) - 2025-05-14

### 🚀 Features

- Update area count and title length to support more data - ([2440fb4](https://code.ornl.gov/research-enablement/acorn/commit/2440fb4141d09f521bcce5d03ebd2c42efff24e9))
- Add TargetMode to xml.rel parsing; Add todo comments for next steps with pptx - ([1c3c42f](https://code.ornl.gov/research-enablement/acorn/commit/1c3c42f9830479987cdd6ea541e9c4b28e4c54bf))
- Remove pptx export pandoc code, add pasthru to verify extract and save capability - ([78ffec8](https://code.ornl.gov/research-enablement/acorn/commit/78ffec81ed29193105c47952d050591066d4694c))
- Cleanup powerpoint archive function - ([6155fdd](https://code.ornl.gov/research-enablement/acorn/commit/6155fdd344350b2b5744872d610658c7e507bf5e))
- Initial creation of archive function; Refine and extend get_all_files - ([3d81129](https://code.ornl.gov/research-enablement/acorn/commit/3d811297622028a29f1f037433d91a624137405e))
- Add extract to .gitignore file for developer experience - ([93a66bb](https://code.ornl.gov/research-enablement/acorn/commit/93a66bb149ec7bdcbfadeefb7f74a1552dc3a407))
- Add capability for parsing xml.rels files - ([cbe7917](https://code.ornl.gov/research-enablement/acorn/commit/cbe7917c5cefc33838a0ef61704da2050dfabcdc))
- Add nanoid and refine code - ([04c1412](https://code.ornl.gov/research-enablement/acorn/commit/04c1412342f141129aed5165db486dc45b187ef1))
- Create powerpoint module with extract function - ([4942bba](https://code.ornl.gov/research-enablement/acorn/commit/4942bba5e389599364c25b19c2e473e85f98c8d1))
- Update dependencies; Make duct workspace dependency - ([1c45fea](https://code.ornl.gov/research-enablement/acorn/commit/1c45fea44b048d834d36749a38bf48f824425bed))
## [0.1.17](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.15..v0.1.17) - 2025-05-09

### 🚀 Features

- Refine accept entries - ([a00c108](https://code.ornl.gov/research-enablement/acorn/commit/a00c108f45cf5eaf115408ab3ba419c976facb2d))
- Update facility character count; Add CAST acronym - ([951f66d](https://code.ornl.gov/research-enablement/acorn/commit/951f66d367241bdbee8e6900421a6d40b3526c84))

### 🐛 Bug Fixes

- *(tests)* Update facility validation test - ([5d14d10](https://code.ornl.gov/research-enablement/acorn/commit/5d14d1063c6e4fa4b9647de7cd36836a2f44e10c))
## [0.1.15](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.14..v0.1.15) - 2025-05-08

### 🚀 Features

- Use regex format for new accept entries; Add LIDAR acronym - ([13a0db4](https://code.ornl.gov/research-enablement/acorn/commit/13a0db4b057957fff7abb76e814540a351b20f26))

### 🐛 Bug Fixes

- *(ci)* Constrain running docs job only when rs files change - ([323cdfc](https://code.ornl.gov/research-enablement/acorn/commit/323cdfc16c3d4db8c621d22b634936cde7d3c9f1))
## [0.1.14](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.13..v0.1.14) - 2025-05-08

### 🐛 Bug Fixes

- Update default FileFormat value - ([0e59877](https://code.ornl.gov/research-enablement/acorn/commit/0e59877ce923f8e1f8e1f16e58e30eab0c6a3985))
## [0.1.13](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.12..v0.1.13) - 2025-05-07

### 🚀 Features

- Add acronyms for TASTI-GRID - ([1010a31](https://code.ornl.gov/research-enablement/acorn/commit/1010a31dbe1273fe6dcbc4d85a224d4c51dad319))
- Add PPT output scaffold to CLI interface - ([349fca0](https://code.ornl.gov/research-enablement/acorn/commit/349fca00dd36e340a3282bac673c7ed118833e9c))
- Refine cli code structure in support of powerpoint support - ([d1f6549](https://code.ornl.gov/research-enablement/acorn/commit/d1f65494cb2b4ca3dbed6098bd13fb9b8ddfeef3))

### 🚧 In Progress

- Initial connection of RAD to PPTX with bucket data - ([fe55d31](https://code.ornl.gov/research-enablement/acorn/commit/fe55d31290258ce4b1ffe58005bbc89f3b93dfda))
## [0.1.12](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.11..v0.1.12) - 2025-05-04

### 🚀 Features

- Add PDF header and footer parameterization based on affiliation - ([577ab86](https://code.ornl.gov/research-enablement/acorn/commit/577ab86ee5918d42a5d36394ddf547a114f9bf81))
- Improve research activity format and add tests - ([24fde3d](https://code.ornl.gov/research-enablement/acorn/commit/24fde3d9f46fecd9371695a0695b47d970c6ca1c))
- Add affiliation resolution to format command - ([b9cc8b6](https://code.ornl.gov/research-enablement/acorn/commit/b9cc8b6a754e2f52669ef57e87bf8d574723cd4f))
- Add contactPoint.affiliation to schema - ([8158d0d](https://code.ornl.gov/research-enablement/acorn/commit/8158d0daec51805d5b1e3a996c4e082a9763c570))
- Add get_nearest capability - ([977ae9f](https://code.ornl.gov/research-enablement/acorn/commit/977ae9fe27c590ea9168c27ac2a8bfe891c4cb8e))
- Add tasti-grid sponsors - ([b1a9a2e](https://code.ornl.gov/research-enablement/acorn/commit/b1a9a2ef65295ea60d17c11edca2f058f7762bbb))
- Add --schema option to export command to export JSON schema - ([0604207](https://code.ornl.gov/research-enablement/acorn/commit/0604207c68b8ebd197b9840d85d26bdcee76eb1e))
- Add --ignore capability to check and format - ([2958b5b](https://code.ornl.gov/research-enablement/acorn/commit/2958b5bce043acbe569c05bec62c4da3771f1a95))
- Refine logic in check command code - ([f7ff5a4](https://code.ornl.gov/research-enablement/acorn/commit/f7ff5a4bc649f275927eac241991e237a086e3a7))

### 🐛 Bug Fixes

- Restore vscode peacock settings - ([2936fb3](https://code.ornl.gov/research-enablement/acorn/commit/2936fb30fa3dc3f8981d4e655cd3b032d61abb40))

### 📚 Documentation

- Improve research activity documentation - ([73734f9](https://code.ornl.gov/research-enablement/acorn/commit/73734f9f31f648138a8126009f65be29d20d08b1))
- Add words to sponsors docs - ([8bcafc5](https://code.ornl.gov/research-enablement/acorn/commit/8bcafc54a38a162f2c71225758e5c32590e361cd))

### 🧪 Testing

- Full coverage for research activity format function - ([d56f6a0](https://code.ornl.gov/research-enablement/acorn/commit/d56f6a0ec362b8f76331e2483237da0853846581))
## [0.1.11](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.10..v0.1.11) - 2025-04-28

### 🐛 Bug Fixes

- *(ci)* Remove test dep from docs job - ([4399bee](https://code.ornl.gov/research-enablement/acorn/commit/4399bee16630fc3a9578a0246ad698836a93cde7))
- *(ci)* Make test run on cargo.toml change - ([b1eddc2](https://code.ornl.gov/research-enablement/acorn/commit/b1eddc2f04e487b09a95858192fd2bfcbaf24d52))
- *(ci)* Make upload-binaries depend on publish-release - ([16b0d10](https://code.ornl.gov/research-enablement/acorn/commit/16b0d1000da220d0211a314df2661af479d3d169))
- *(ci)* Remove rule that was too restrictive - ([fb1bf8c](https://code.ornl.gov/research-enablement/acorn/commit/fb1bf8c250a40058910e13e325de0526e121d9de))
## [0.1.10](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.9..v0.1.10) - 2025-04-28

### 🚀 Features

- Use DOE- prefix for sponsor offices - ([d29de0d](https://code.ornl.gov/research-enablement/acorn/commit/d29de0d57b5f9995b2523d335c22bc19e8e8326f))

### ⚙️ Miscellaneous Tasks

- Update rules for when test is run - ([c6acfb2](https://code.ornl.gov/research-enablement/acorn/commit/c6acfb202bb9dd94faa5e5b1ea4b666c55e9a18c))
## [0.1.9](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.8..v0.1.9) - 2025-04-28

### 🚀 Features

- Initial organization graph model capability - ([d29743e](https://code.ornl.gov/research-enablement/acorn/commit/d29743ea28ba2da9cb12aab81377f7a96d4ca0af))
- Add serde-trim and serde-with to trim whitespace and refine option handling - ([252633a](https://code.ornl.gov/research-enablement/acorn/commit/252633a40a21a4ffe86b445ccec5b9f5a468dd6d))

### 🐛 Bug Fixes

- Observe optional serializing for partners and sponsors - ([089fab0](https://code.ornl.gov/research-enablement/acorn/commit/089fab094aa3bb46ba4deb7b306c05a4793d8054))
## [0.1.8](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.7..v0.1.8) - 2025-04-27

### 🚀 Features

- Improve check output immensely - ([c5d5815](https://code.ornl.gov/research-enablement/acorn/commit/c5d581534e661b2654a391c6254dd1ef5da4df61))
- Doctor installed software check refinement - ([a613487](https://code.ornl.gov/research-enablement/acorn/commit/a613487cd768b82ceaa09e317381bfea3d08488d))
- Initial doctor software check print - ([13e20fd](https://code.ornl.gov/research-enablement/acorn/commit/13e20fd4425f08d288da6a4b1f1a32e4a1536176))
- Add code to get CLI app version - ([c9b6ea9](https://code.ornl.gov/research-enablement/acorn/commit/c9b6ea9c0e47c31a7a5ac521b7fd456aea9e17ce))
- Add OSTI data to partners csv asset - ([143289a](https://code.ornl.gov/research-enablement/acorn/commit/143289a70debe848821820064595f29d2809dc89))
- Add OSTI data to sponsors csv asset - ([cdec59a](https://code.ornl.gov/research-enablement/acorn/commit/cdec59a9483c28c2709539a83670eb3f03347a27))
- Refine ignoring files during download command - ([f73744b](https://code.ornl.gov/research-enablement/acorn/commit/f73744be6c17af0ccd78faa8f35b9df451f975ee))
- Stop downloading .DS_Store - ([2c846a8](https://code.ornl.gov/research-enablement/acorn/commit/2c846a83323dc111433763c0b99ed01ca1b625d5))

### 🐛 Bug Fixes

- Add hydrosource to accept - ([9fe213b](https://code.ornl.gov/research-enablement/acorn/commit/9fe213b6fe8756e23582b7734b248fd46cb6822c))
- Update validation messages to match updated constants - ([a0b728a](https://code.ornl.gov/research-enablement/acorn/commit/a0b728accf122c41dccaee4ce48fa530955dd70e))
## [0.1.7](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.6..v0.1.7) - 2025-04-24

### 🚀 Features

- Add programming languages from OSTI data to technology csv asset - ([64a387c](https://code.ornl.gov/research-enablement/acorn/commit/64a387c518e37ceafe0bb44c83a1c030d98a5cce))
- Update constants based on central comms feedback - ([a5d6452](https://code.ornl.gov/research-enablement/acorn/commit/a5d6452cdf0b11b3af459d5141cffc8ebd082f7c))
- Cull Organization impl ISO using graph model for resolving organization data - ([3c83838](https://code.ornl.gov/research-enablement/acorn/commit/3c83838997bb76b500bb454014f48027d084836b))
- Abstract table printing into function - ([37325bd](https://code.ornl.gov/research-enablement/acorn/commit/37325bdd8a29d90c2353cec7873a42aac58ca0fe))
- Add dependencies for linked data and future tasks - ([0b10421](https://code.ornl.gov/research-enablement/acorn/commit/0b104210d320d6271075df5624a23ce389618a38))

### 🐛 Bug Fixes

- *(docs)* Update organization to point to organization - ([e2b1cb6](https://code.ornl.gov/research-enablement/acorn/commit/e2b1cb624ce193161e87d61ebd307ba60cc9ebff))
- *(docs)* Use correct link URLs for structs - ([a61d19b](https://code.ornl.gov/research-enablement/acorn/commit/a61d19ba583c51e94b8be2063c39f72602a24ff8))
- *(docs)* Use correct link syntax for structs - ([67839fc](https://code.ornl.gov/research-enablement/acorn/commit/67839fc8c5dfe7f6b4862c45be0a7f3c773a84ba))
- Add QIS to accept - ([5f0a4be](https://code.ornl.gov/research-enablement/acorn/commit/5f0a4be22a21b6c02ef46418dab170b6ad9ab2ec))
- Remove test code - ([d461ae2](https://code.ornl.gov/research-enablement/acorn/commit/d461ae233b38e057fde11f2d80c750be22b7fee4))

### 📚 Documentation

- Improve help and linked data compliance - ([30ce36a](https://code.ornl.gov/research-enablement/acorn/commit/30ce36adc32b06e904dc9c6bad5e0f91fba25a6a))

### 🚧 In Progress

- Add --ignore option to check command interface - ([2300681](https://code.ornl.gov/research-enablement/acorn/commit/23006815f5bd7a322bdd916e05fdd9750fa1dce9))
- More preparation for parameterizing on organization; add header and footer images - ([3972fb7](https://code.ornl.gov/research-enablement/acorn/commit/3972fb78d81d296f99d5d4725d8b80971addb9a4))
- Preparation for parameterizing on organization - ([ba6e261](https://code.ornl.gov/research-enablement/acorn/commit/ba6e2610d0a1416d6f1c7d0dbb2be3950e197873))

### 🤓 Developer Experience

- Add notebooks to project for dev - ([83ae021](https://code.ornl.gov/research-enablement/acorn/commit/83ae0211ae41813b335c38ca31724cafe01576d1))
## [0.1.6](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.4..v0.1.6) - 2025-04-14

### 🚀 Features

- Dynamically add acronyms, sponsors, and partners to vale accept - ([b206f87](https://code.ornl.gov/research-enablement/acorn/commit/b206f87d875c9b33f1f61c0991060ec9d86aa6bd))
- Add to accept and add alternative name QSC - ([5b6c9c5](https://code.ornl.gov/research-enablement/acorn/commit/5b6c9c5472d541a79fe549aff552e77dba72c1c4))
- Add hackathon to accept - ([5eae104](https://code.ornl.gov/research-enablement/acorn/commit/5eae104b6538837d38276c8cd6b38a78efac164f))
- Remove empty alternativeNames from organization.json - ([dd628e4](https://code.ornl.gov/research-enablement/acorn/commit/dd628e49d865b772d3c3a40eada511fdac5e9589))
- Add items to accept; Create acronyms.csv - ([2fa1fed](https://code.ornl.gov/research-enablement/acorn/commit/2fa1fed65f973133ae8e12000fa513d916002e19))
- Add readability tests with nearly 100 percent code coverage - ([d27b266](https://code.ornl.gov/research-enablement/acorn/commit/d27b266b8c320077f5024aff359eeb57e8649512))
- Refactor readability into module - ([5e668f5](https://code.ornl.gov/research-enablement/acorn/commit/5e668f5a2baf54921a66a3d6bc6e1d45578cfe26))
- Initial syllable count capability - ([1fa431a](https://code.ornl.gov/research-enablement/acorn/commit/1fa431a5af1f3f269fb33df6a33d2e9aaf1155b9))
## [0.1.4](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.3..v0.1.4) - 2025-04-11

### 🚀 Features

- Update accept list for FFESD data - ([341115d](https://code.ornl.gov/research-enablement/acorn/commit/341115df42a6f197497459fe07ed303db53d5549))
## [0.1.3](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.2..v0.1.3) - 2025-04-11

### 🚀 Features

- Improve linked data compliance of schema - ([f038f59](https://code.ornl.gov/research-enablement/acorn/commit/f038f59e4b2c10181aa89bf846abc387741fc282))
- Initial creation of get_singular_form; Initial structure for get_syllable_count - ([d1fdd0c](https://code.ornl.gov/research-enablement/acorn/commit/d1fdd0c67711af1273594cf920a791657be2b9f9))

### 📚 Documentation

- Fix link in doc and add more links - ([0318a38](https://code.ornl.gov/research-enablement/acorn/commit/0318a3831d1166aafa454764174df6df5d1e4f79))
- Add link to full list of keywords to Keyword type - ([4a9a2b8](https://code.ornl.gov/research-enablement/acorn/commit/4a9a2b8a23a9929eac99e717f47c5980404c152f))
- Make Keyword type public - ([8c1100a](https://code.ornl.gov/research-enablement/acorn/commit/8c1100a58ccac603d6c2a46fcce80fcb9a209cbb))

### ⚙️ Miscellaneous Tasks

- Clean up config - ([8d14ace](https://code.ornl.gov/research-enablement/acorn/commit/8d14ace8312517d5fa00ebdd5829389d34728f99))
- Make build-release only run when version is incremented - ([85c3e11](https://code.ornl.gov/research-enablement/acorn/commit/85c3e116be8de5178fa641e28008a924e5c44cc8))

### 🚧 In Progress

- Handle syllable counting edge cases - ([32ec660](https://code.ornl.gov/research-enablement/acorn/commit/32ec660b709a7861a66c74ddc0fd5aafb94bce0b))
- Initial creation of doctor structs for system software - ([cd28ce0](https://code.ornl.gov/research-enablement/acorn/commit/cd28ce043105baa4c9fa3c64c94973ef72178d78))
## [0.1.2](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.1..v0.1.2) - 2025-04-08

### 🚀 Features

- Add commit option to export command - ([367c712](https://code.ornl.gov/research-enablement/acorn/commit/367c712e9c0929fa2610dc5a220099fcfda0154b))
- Decrease logging of vale config to trace - ([c618cfb](https://code.ornl.gov/research-enablement/acorn/commit/c618cfbc3f754bee61ecaa9ad6b170845d426970))

### 🐛 Bug Fixes

- Add trace to file imports - ([5bdc0b2](https://code.ornl.gov/research-enablement/acorn/commit/5bdc0b290cf611ba41520666e5da90897060626b))
## [0.1.1](https://code.ornl.gov/research-enablement/acorn/compare/v0.1.0..v0.1.1) - 2025-04-05

### 🐛 Bug Fixes

- Bump patch version - ([0861ecb](https://code.ornl.gov/research-enablement/acorn/commit/0861ecb96302712fe39dadf4864d723b78c32034))
## [0.1.0](https://code.ornl.gov/research-enablement/acorn/compare/v0.0.6..v0.1.0) - 2025-04-05

### 🚀 Features

- Improve schema JSON-LD compliance - ([8050f3e](https://code.ornl.gov/research-enablement/acorn/commit/8050f3e3edda40d8b778e07502178ae3b1fb1fae))
- Add commit option to format sub command and normalize get_all_files arguments - ([22e3a19](https://code.ornl.gov/research-enablement/acorn/commit/22e3a1915d00c8e3ec074d258a5fe5e6e6ac73ed))
- Add ability to pass paths to check from a git commit hash - ([ad6253e](https://code.ornl.gov/research-enablement/acorn/commit/ad6253eb84124dbc11b7dc212d193ac0649e88f0))
- Update keyword list; add tests - ([3ed7e6a](https://code.ornl.gov/research-enablement/acorn/commit/3ed7e6adc1a479a581be7a316d3d7bf6ea36e0ae))
- Add captions to PDF export targets - ([21bd45f](https://code.ornl.gov/research-enablement/acorn/commit/21bd45f9e53786f484c9b10dbddf2ea796916b41))
- Add alias for topsecret - ([db5dd47](https://code.ornl.gov/research-enablement/acorn/commit/db5dd470436cdf0f8946e7a1e9dcf7748538419b))
- Add classification and additional_type to schema - ([6521790](https://code.ornl.gov/research-enablement/acorn/commit/6521790e6b198640f32fb44346e061b4a4cb7a06))

### 🐛 Bug Fixes

- Remove broken tests and refactor tests file modules as folder modules - ([221ce26](https://code.ornl.gov/research-enablement/acorn/commit/221ce264fcbf9b73b3cb3c4ce1e3ec68dfd7f67d))

### 📚 Documentation

- Add more keywords after reviewing core capabilities - ([535984e](https://code.ornl.gov/research-enablement/acorn/commit/535984e7ee2ef349e0ce51576c84409268095de8))
- Refine keywords inline docs - ([b505674](https://code.ornl.gov/research-enablement/acorn/commit/b505674e07642224487e15d5cbc01d3cac755dc1))

### 🧪 Testing

- Clean up doctor command tests - ([59e88f1](https://code.ornl.gov/research-enablement/acorn/commit/59e88f1707d5ee9dd7ab5b4539f85af091bc8b95))
- Add recommendations from warp AI - ([b4f1bf0](https://code.ornl.gov/research-enablement/acorn/commit/b4f1bf064c4d7cb01291931fc0278d5cc37c7b37))

### 🚧 In Progress

- Add docs to ClassificationLevel - ([84ca184](https://code.ornl.gov/research-enablement/acorn/commit/84ca184d4f96bc5eca38e942f2ffb8d9810ad0fb))
## [0.0.6](https://code.ornl.gov/research-enablement/acorn/compare/v0.0.5..v0.0.6) - 2025-04-02

### 🚀 Features

- Add TRL to meta schema - ([8474921](https://code.ornl.gov/research-enablement/acorn/commit/847492197ad8a7986c7a2ec23176582ce41eac1d))
- Add validation for meta.graphics.href - ([e0cebe3](https://code.ornl.gov/research-enablement/acorn/commit/e0cebe35fa3c87e10d6c43dd26317ae3da73d00c))
- Move validation maximum values to constant module - ([398fcb3](https://code.ornl.gov/research-enablement/acorn/commit/398fcb3acdd3f6d8802d4d83d96894fc12236cc9))
- Add minimum values to validation of multiple fields - ([e2b73d6](https://code.ornl.gov/research-enablement/acorn/commit/e2b73d6c47872bbcdefb48dfad64a2cd3b64d302))
- Add WPP to organizations - ([60e6115](https://code.ornl.gov/research-enablement/acorn/commit/60e6115fa4a9b1abc7eda024b58754ceb6d2764e))

### 🐛 Bug Fixes

- Restore subtitle max length to 100 - ([2944b7b](https://code.ornl.gov/research-enablement/acorn/commit/2944b7bef6e1aea5cd9933184d6fdc840febb145))
## [0.0.5](https://code.ornl.gov/research-enablement/acorn/compare/v0.0.4..v0.0.5) - 2025-03-26

### 🚀 Features

- Refine resolve method; Add meta.related attribute - ([ff14deb](https://code.ornl.gov/research-enablement/acorn/commit/ff14deba37ad9170b7509e93240d154551a9f359))
- Refine fuzzy matching, use same function for keywords, partners, sponsors, and technology - ([8c1fee1](https://code.ornl.gov/research-enablement/acorn/commit/8c1fee182f08c91480cbd6696a230d5f69b9c586))
- Refactor buckets config for simpler interface - ([d3b88fa](https://code.ornl.gov/research-enablement/acorn/commit/d3b88fa146e2fb0a552b78957d60f3dae5df5032))
- Update lib manifest - ([8b4fcc7](https://code.ornl.gov/research-enablement/acorn/commit/8b4fcc71a8c19d3c2a9e37c2be528ad52c51bb8d))
- Remove unused dependencies with machete - ([14c58ad](https://code.ornl.gov/research-enablement/acorn/commit/14c58ad9dd96509bb2284da66a72f01bc859ad1b))
- Refine format JSON diff output - ([ac9f453](https://code.ornl.gov/research-enablement/acorn/commit/ac9f4530b5247b252d79c218a829d586e6fdfb26))
- Add diff output for format command - ([ed528cf](https://code.ornl.gov/research-enablement/acorn/commit/ed528cf1392760f87683107f5763bf6aeecd6d90))
- Add acronyms and words to accept file - ([a2e5f01](https://code.ornl.gov/research-enablement/acorn/commit/a2e5f014e5afa2acd9ac9f9394a0667cef506aff))
- Remove debug from release profile - ([df6a5a1](https://code.ornl.gov/research-enablement/acorn/commit/df6a5a1d1031fd5ddd7e6c01df5676c73c66d615))
- Improve manifest metadata and documentation - ([96b885f](https://code.ornl.gov/research-enablement/acorn/commit/96b885f6910e135fbd08a9af54a98545e98b95c5))
- Refactor keyword resolve to use resolve_from_csv - ([50adbb3](https://code.ornl.gov/research-enablement/acorn/commit/50adbb3857def29147d997a809f506b6960a320b))
- Improve technology resolution - handle no matches case - ([3a6a292](https://code.ornl.gov/research-enablement/acorn/commit/3a6a292ae788fdf9a814fa5c2c4d7a381e7f8fc4))
- Update schema ISO removing highlight as schema type - ([78e5210](https://code.ornl.gov/research-enablement/acorn/commit/78e521091b60465263dadc862e5af517d18d928f))
- Run make lint - ([7573025](https://code.ornl.gov/research-enablement/acorn/commit/75730256710e9e27f775219e373e207f7b0ee6fb))
- Move crates used for testing to dev-dependencies - ([13d65f6](https://code.ornl.gov/research-enablement/acorn/commit/13d65f6a9a97c4df2e7ae490e4341685bf1cee68))
- More improvements to download output - ([e8f1544](https://code.ornl.gov/research-enablement/acorn/commit/e8f1544708eda69a99c1281487329b96b6aaa1dd))
- Improve download command output - ([5195f3c](https://code.ornl.gov/research-enablement/acorn/commit/5195f3cae48f127d52b743746348bf75093db489))
- Lint toml files - ([1b7e27e](https://code.ornl.gov/research-enablement/acorn/commit/1b7e27ef2838608069e15363e88b74c4f0a1fb0a))
- Move cross config into Cargo.toml; Use nextest for testing; Refine make tests - ([972eb2b](https://code.ornl.gov/research-enablement/acorn/commit/972eb2bd7d3fb96731c2724de6333996f76deeeb))

### 🐛 Bug Fixes

- *(ci)* Temp add default target after removing cross.toml - ([cf1e478](https://code.ornl.gov/research-enablement/acorn/commit/cf1e478c881311cc53cede20a6de366a559234d8))
- *(ci)* Restore test-ci make task with correct name - ([d2adf7e](https://code.ornl.gov/research-enablement/acorn/commit/d2adf7e85ed8cc46cd2f67fabfd3e1943a5897ea))
- *(ci)* Restore test-ci make task - ([1ff06f8](https://code.ornl.gov/research-enablement/acorn/commit/1ff06f83d4a8d4a159dd49fd0d04798b9f65e676))
- Bump version - ([d80ab48](https://code.ornl.gov/research-enablement/acorn/commit/d80ab48f70e2b4e9c37cc1f9b7e9d86290abb16c))
- Refactor some manifest attributes to clear warnings - ([326f7a8](https://code.ornl.gov/research-enablement/acorn/commit/326f7a8af73514f8c51b8e504f96e475d92841dc))
- Remove openssl dependencies - ([60a3f5b](https://code.ornl.gov/research-enablement/acorn/commit/60a3f5ba9bf14a161383ead2b237f05d67aa7bfb))
- Update serialization for new optional attributes; Replace ampersands in organization fuzzy matching - ([b927abc](https://code.ornl.gov/research-enablement/acorn/commit/b927abce72f35b79d7f182180965ab16d3b670f0))
- Add phony tasks to makefile - ([d3b393b](https://code.ornl.gov/research-enablement/acorn/commit/d3b393b6ae33ef7fe7f9669774b539958c3a2296))

### 📚 Documentation

- Add maintenance metadata to manifest - ([c40809f](https://code.ornl.gov/research-enablement/acorn/commit/c40809fff5599455eb8bacb94d760d809c98b526))
- Refine doc comments - ([96caf4d](https://code.ornl.gov/research-enablement/acorn/commit/96caf4d20f9ee7706b49d15356820d95743eba1c))
- Updates to metadata schema - ([36ee9a8](https://code.ornl.gov/research-enablement/acorn/commit/36ee9a8777ac9159862de813fd93bc4db9c33788))
- Minor tweaks and improvements - ([f5f84b2](https://code.ornl.gov/research-enablement/acorn/commit/f5f84b2e985f25ff8f94b05e0af4d8e39ea41f30))
- Add doc comments to metadata struct - ([f55cf54](https://code.ornl.gov/research-enablement/acorn/commit/f55cf54379c02913b37ef29d8297176f3a0dc1f2))

### 🧪 Testing

- Add some tests for bucket code - ([8512319](https://code.ornl.gov/research-enablement/acorn/commit/8512319720da9b3e66fb4f2e9db444fbada35b52))
- Update coverage - ([72c069c](https://code.ornl.gov/research-enablement/acorn/commit/72c069c8abb5cbd64c1f7d4f48c06a0f85879fb0))
- Update coverage - ([2d5ff2d](https://code.ornl.gov/research-enablement/acorn/commit/2d5ff2da79c5424d93063527263b2be5b636b18e))

### 🚧 In Progress

- Add dependencies and add initial code for JSON diff output during format - ([168b6de](https://code.ornl.gov/research-enablement/acorn/commit/168b6dee7930872bf43cef869cc816564f3e9c6b))
## [0.0.4](https://code.ornl.gov/research-enablement/acorn/compare/v0.0.3..v0.0.4) - 2025-03-07

### 🚀 Features

- Bump version to trigger release - ([6426b4d](https://code.ornl.gov/research-enablement/acorn/commit/6426b4d5ecb76cef9835338eb3d264051bf4b356))
- Add superfluidity to accept list - ([b689c16](https://code.ornl.gov/research-enablement/acorn/commit/b689c1665020ffdcac4134072aeafee8616ac88c))
- Make subtitle optional and add words to accept - ([891a22d](https://code.ornl.gov/research-enablement/acorn/commit/891a22d8b280d2e309b206e2ae4f06bae3bbd911))
- Improve download command handling of empty buckets - ([9006525](https://code.ornl.gov/research-enablement/acorn/commit/9006525482bfa38354e234c3d06772cdafefeb0a))
- Center progressbar template counts - ([c791aa4](https://code.ornl.gov/research-enablement/acorn/commit/c791aa4a3a044fba3b0b4dae84871eb8950a7726))
- Add REDC to organization.json - ([2e036eb](https://code.ornl.gov/research-enablement/acorn/commit/2e036eb58d6b317d60bf6111774f657153d438ba))
## [0.0.3] - 2025-03-05

### 🚀 Features

- *(ci)* Bump version; add manual trigger to docs generation - ([19fec68](https://code.ornl.gov/research-enablement/acorn/commit/19fec680c87629ad822d983165929fdd1c85c69b))
- *(ci)* Add Cross configuration - ([72cddea](https://code.ornl.gov/research-enablement/acorn/commit/72cddeaa4eb509493a8cba6b96527e8878bebee2))
- *(ci)* Configure doc site and binary release - ([a4682db](https://code.ornl.gov/research-enablement/acorn/commit/a4682db6f25b402f02484bb3850fe2b5b68bf0e1))
- Make check return failure code on error; add exit-on-first-error - ([d2c40ff](https://code.ornl.gov/research-enablement/acorn/commit/d2c40ff24870168a972b4a125d2257ea9b222471))
- Add partners attribute to meta - ([fb59e83](https://code.ornl.gov/research-enablement/acorn/commit/fb59e838e73d05291bfd0fbc8735b9da0d64b0e8))
- Move some consts to constants module - ([322e67d](https://code.ornl.gov/research-enablement/acorn/commit/322e67d296e9e7a8102ec532f90bd009abe00309))
- Make sections.facilities optional - ([adb658a](https://code.ornl.gov/research-enablement/acorn/commit/adb658a2e2d1ceef3565fc280a1d7d3d8619ac7c))
- Add eserde json feature; reorder lib dep list - ([90c786e](https://code.ornl.gov/research-enablement/acorn/commit/90c786eb909596f0dafc496e7fd9d014829fde77))
- Add eserde dep - ([92fbd92](https://code.ornl.gov/research-enablement/acorn/commit/92fbd922bd1d660b034ec4ee9895fadbcb9516f9))
- Improve highlight output - ([be1f012](https://code.ornl.gov/research-enablement/acorn/commit/be1f0122fd890d22ee238d3cf3cbccd51cdcec9b))
- Add ror attribute to organization schema; update organization.json - ([1395f40](https://code.ornl.gov/research-enablement/acorn/commit/1395f40ba59926fb25a71647a81cfd5f95a78c96))
- Move validation functions into separate submodule; Add tests for Metadata - ([85f2c5c](https://code.ornl.gov/research-enablement/acorn/commit/85f2c5cdcd78edc3b73abcced5efb68d92f2db0c))
- Remove playwright dep from lib - ([528ea0e](https://code.ornl.gov/research-enablement/acorn/commit/528ea0e35da1f8fa38595c78b574e8fd585ade6e))
- Move creat_pdf to export module - ([061f9de](https://code.ornl.gov/research-enablement/acorn/commit/061f9ded30ba44a8666f9002fcabe8015c232d21))
- Refactor commands into submodules; initial highlight output connection - ([5e44f5a](https://code.ornl.gov/research-enablement/acorn/commit/5e44f5a16f7ef64342089b2f44dca73ac0fd818d))
- Rename meta.website to meta.websites - ([0335e86](https://code.ornl.gov/research-enablement/acorn/commit/0335e864ce536d3a3fcd545fe8c3f6b415f5b9ea))
- Make meta.website a Vec of Websites - ([62e497a](https://code.ornl.gov/research-enablement/acorn/commit/62e497a45d1f98b5887e8df045d34238a670abaf))
- Add raid and ror attributes to meta struct - ([a69b633](https://code.ornl.gov/research-enablement/acorn/commit/a69b633877ff14a5906a084de31ede29af8e6170))
- Move consts to constants.rs - ([3021520](https://code.ornl.gov/research-enablement/acorn/commit/3021520c3942fbc8069e521b435ce23b83560aae))
- Move sponsors and technology lists to assets constants folder - ([da524a0](https://code.ornl.gov/research-enablement/acorn/commit/da524a073fa1a4efdc4da48156e0ee1046c5253c))
- Refine organization methods; add petgraph dep - ([3155e39](https://code.ornl.gov/research-enablement/acorn/commit/3155e39d591d33927c162eb8450df72da491b3b7))
- Move head and footer instantiation into html templates; enhance Organization navigation - ([3071155](https://code.ornl.gov/research-enablement/acorn/commit/3071155c85b28c979a39a8a2bd79ebce1f083dcb))
- Parameterize path to CSS stylesheet in template - ([c76ddf0](https://code.ornl.gov/research-enablement/acorn/commit/c76ddf0bbf13f27802cc8cdf13fca42f7b763379))
- Rename contact.division -> contact.organization - ([25c9153](https://code.ornl.gov/research-enablement/acorn/commit/25c9153c1e7b5f79b0be464ff8f9af35fa63b6d4))
- Create constants module; Refine dependencies - ([b5ea37a](https://code.ornl.gov/research-enablement/acorn/commit/b5ea37a0cac73def24957db91fbeb7d38624eb2b))
- Make doctor checks opt in - ([949ac56](https://code.ornl.gov/research-enablement/acorn/commit/949ac56adab8d5fba68ca92ad10221aaf2b83894))
- Improve doctor network information output - ([4f892bc](https://code.ornl.gov/research-enablement/acorn/commit/4f892bc44881c8dc471a823950bdc2d497157f7f))
- Add skip for doctor; refine doctor code - ([7148489](https://code.ornl.gov/research-enablement/acorn/commit/714848975303c1c78f4239d3a069cac1b8ea6ad3))
- Add table output to doctor command - ([ebad05f](https://code.ornl.gov/research-enablement/acorn/commit/ebad05f0f79fd6fb1fb2df310831db782c5511ed))
- Add reject.txt to vale analysis - ([ac543a6](https://code.ornl.gov/research-enablement/acorn/commit/ac543a6429c92c368102421bc62c6be1296a85e1))
- Make static analyzer struct more extensible; shrink fact sheet pdf research focus font size - ([10f3b84](https://code.ornl.gov/research-enablement/acorn/commit/10f3b848f6f01735f1a16302f970fd259042c99a))
- Use system vale command when available - ([43e4a99](https://code.ornl.gov/research-enablement/acorn/commit/43e4a994d7c91afc119576466a9ac327be157699))
- Add test_command function for checking commands - ([e5e9bd1](https://code.ornl.gov/research-enablement/acorn/commit/e5e9bd1ffb4bf053bd44cf82bc484e55c5f2d7c4))
- Integrate directory check into get_all_files - ([4a50296](https://code.ornl.gov/research-enablement/acorn/commit/4a50296551bf07bbc24f5584a902e7ce510206c6))
- Refactor export functions for clarity and ease of use - ([b0da8b4](https://code.ornl.gov/research-enablement/acorn/commit/b0da8b4c2739c7fe353de61997d7c7a901d482c4))
- Use get_all_files in export command; add banner images; abstract getting stylesheet - ([8341981](https://code.ornl.gov/research-enablement/acorn/commit/8341981aa7171095b2aa6ea5e9266b8cecfc9ec1))
- Create Style asset - ([2c3aa32](https://code.ornl.gov/research-enablement/acorn/commit/2c3aa3274ff6f62e737d96b942c8de32b62ad735))
- Update help docs and remove a TODO - ([01d1144](https://code.ornl.gov/research-enablement/acorn/commit/01d1144c69b890fcb58c6031e29d2d9c9273d2ed))
- Initial format capability - ([489b88b](https://code.ornl.gov/research-enablement/acorn/commit/489b88b728c5cdbfadc1a39b8dff3ff01bf94d09))
- Add skip capability to check command; Refine fuzzy matching code with types - ([5c746ee](https://code.ornl.gov/research-enablement/acorn/commit/5c746ee1817d17a758f92474239fc9b548f6aecf))
- Refactor check and analyze for less code in main - ([a5f4271](https://code.ornl.gov/research-enablement/acorn/commit/a5f4271a6789a8eebbcb2323d8f93f0413c6f91a))
- Run lint and generate code coverage - ([256f001](https://code.ornl.gov/research-enablement/acorn/commit/256f0013962f1d09fa09c3c8563ddb5ece4b9a1d))
- Use get_all_files function in main for much win - ([a205ade](https://code.ornl.gov/research-enablement/acorn/commit/a205ade1679dc60ba04ee25c5725256f75214092))
- Add words to vocab accept; rename highlight methods to technical - ([28916f1](https://code.ornl.gov/research-enablement/acorn/commit/28916f1ccb42670b5bb22118a55c034e6d920b3c))
- Make get_all_files work and add tests; update test data - ([9237773](https://code.ornl.gov/research-enablement/acorn/commit/9237773ab9a21161cff4a69eb3d39c710a8d2037))
- Add org-chart data and struct - ([62481ca](https://code.ornl.gov/research-enablement/acorn/commit/62481ca0346187699446f92fcdcff0e72efd3af1))
- Add total schema error count log - ([50d209e](https://code.ornl.gov/research-enablement/acorn/commit/50d209e1e1703fa3997c3fec049ad242ec43de85))
- Add target size option to export subcommand - ([ac0f7b2](https://code.ornl.gov/research-enablement/acorn/commit/ac0f7b2ea550e7c82c3ddbb12256501b7d582e55))
- Update outcomes validation; Refine tests - ([48c7055](https://code.ornl.gov/research-enablement/acorn/commit/48c70552bf737fa63d221a030d7e08782269a951))
- Refine vale output; add items to vocab accept list - ([104b5bb](https://code.ornl.gov/research-enablement/acorn/commit/104b5bbc9ade9edb352aaffda1eec62b322ce541))
- Minor tweak to error handling - ([1f235cd](https://code.ornl.gov/research-enablement/acorn/commit/1f235cdc53a471be4ad11cb2bfc5de19e7306928))
- Make some TODO comments and add accept.txt asset - ([7ea810a](https://code.ornl.gov/research-enablement/acorn/commit/7ea810acc879603a695e91d6d9da444e1c4f4499))
- Make vale sync more robust - ([c49f7e7](https://code.ornl.gov/research-enablement/acorn/commit/c49f7e7a9ac3ed049675849186839cbb04e1d6c3))
- Save prose to cache project folder and refine error output - ([93cf779](https://code.ornl.gov/research-enablement/acorn/commit/93cf779761b0adb65195a40807a8541c2fd31322))
- Add ability to extract prose from project research data - ([9a4fb91](https://code.ornl.gov/research-enablement/acorn/commit/9a4fb911e31767564ce95085c952c2bfafc66334))
- Refactor location of some functions to make more sense - ([124ffbb](https://code.ornl.gov/research-enablement/acorn/commit/124ffbbef21317e3bcd2adc24d16fecd36f3b8f8))
- Remove base64 crate in favor of consolidating with data-encoding crate - ([128924f](https://code.ornl.gov/research-enablement/acorn/commit/128924fb19ecf562cedab14f631f528385511b78))
- Create Vale struct for encapsulating vale capability - ([6b4c6cf](https://code.ornl.gov/research-enablement/acorn/commit/6b4c6cf3d7023920dbffc1edf1a1ff9cca5b8ee2))
- Update and apply rustfmt configuration - ([519c0d7](https://code.ornl.gov/research-enablement/acorn/commit/519c0d74e03e19772d09f88cefa9690a367eb255))
- Refactor code into cli and lib - ([3d15999](https://code.ornl.gov/research-enablement/acorn/commit/3d159999529f943c23e34da8d628ee425d1d4b28))
- Refactor util module to folder structure - ([9f29f11](https://code.ornl.gov/research-enablement/acorn/commit/9f29f1199448fc74177a8c2c088ef1584f13d63b))
- Consolidate parse into format - ([7360c8d](https://code.ornl.gov/research-enablement/acorn/commit/7360c8d617f979bcb2066544648c607240a20119))
- Add logging for fuzzy matching and fix test data - ([ce8b5fe](https://code.ornl.gov/research-enablement/acorn/commit/ce8b5feaab1cd0af6ebee67d69a957a25f638b09))
- Minor updates to logging; change lint to check in function name - ([f4978f6](https://code.ornl.gov/research-enablement/acorn/commit/f4978f6d255ed42b7d27dca6105d8f596395f3ff))
- Rename lint to check; Add placeholder for format command - ([1ddc683](https://code.ornl.gov/research-enablement/acorn/commit/1ddc6831b71ddee8468ef5043a2df5c12ae94980))
- Add capability to get first graphic href (with default) - ([c3a324c](https://code.ornl.gov/research-enablement/acorn/commit/c3a324c2d706778825a687180d6d83089484551c))
- Parametrize font and image assets - ([da6c5a7](https://code.ornl.gov/research-enablement/acorn/commit/da6c5a70244d6635e130a8fd14f364d61ac90f6f))
- Code cleanup; Place some TODO markers for next steps - ([48bdfde](https://code.ornl.gov/research-enablement/acorn/commit/48bdfde17884fff9f28007afa076d73cb8266f12))
- Add fonts to acorn binary for PDF export - ([d318fcd](https://code.ornl.gov/research-enablement/acorn/commit/d318fcd049bb4cd3d26ee61ea7444b8be7c3b945))
- Add qr code support to PDF export - ([90d5f2d](https://code.ornl.gov/research-enablement/acorn/commit/90d5f2d603f690e17910c86247edfd0ac5e87998))
- Refine download pagination implementation - ([e191a05](https://code.ornl.gov/research-enablement/acorn/commit/e191a0580fbe54f23a97540d79f842a1cd432249))
- Initial addition of download pagination support - ([25150e0](https://code.ornl.gov/research-enablement/acorn/commit/25150e0cbc04a48bbe04b5e265b2d0baf9a77878))
- DRY up some code ISO adding pagination support - ([84856ea](https://code.ornl.gov/research-enablement/acorn/commit/84856eaf6896faaa500c5b51aa235f44e161815e))
- Make download work in parallel - ([c13f80c](https://code.ornl.gov/research-enablement/acorn/commit/c13f80c80ad54c65562c0b3cfc48162cc29ff697))
- Cull unused code and dependencies - ([2d07cc6](https://code.ornl.gov/research-enablement/acorn/commit/2d07cc62eb89dda1987f4c4b3be5804e429b56ef))
- Update download to support new bucket folder structure - ([d40380c](https://code.ornl.gov/research-enablement/acorn/commit/d40380cb467425f2a1ac8f942b68e67622f906c5))
- Add assets to project and load as base64 - ([6a7fed0](https://code.ornl.gov/research-enablement/acorn/commit/6a7fed04a01a41a4fe4ef2d7513ada1c148a05ed))
- Use structs and impls in template submodule; add css multiline string for styling - ([f8baa57](https://code.ornl.gov/research-enablement/acorn/commit/f8baa5771b3eab62257bab2e82208d216312c40e))
- Add template submodule with initial HTML generation capability via export subcommand - ([8c72f8e](https://code.ornl.gov/research-enablement/acorn/commit/8c72f8e90de8a747810dfafd5113219b313d3e17))
- Add attributes to Graphic struct - ([d1b732f](https://code.ornl.gov/research-enablement/acorn/commit/d1b732fef973fa5d14a87d2b10db76908fa0d711))
- Add validation for highlight attributes including DOI - ([c78a224](https://code.ornl.gov/research-enablement/acorn/commit/c78a2243981aa1592e88a3707de5fae6ee73e005))
- Initial addition of highlight schema attributes - ([00d30a6](https://code.ornl.gov/research-enablement/acorn/commit/00d30a68084929cc85db8ae7099c2575667ba3d0))
- Add progress to download subcommand - ([e9d615a](https://code.ornl.gov/research-enablement/acorn/commit/e9d615a0b51e0e1c0ee36a1a27203f621d806e91))
- Style errors in main - ([1c02ff0](https://code.ornl.gov/research-enablement/acorn/commit/1c02ff0408073e3b86e066b749dcbf85c7ed79b8))
- Add tests and improve kebabcase validation - ([ead899d](https://code.ornl.gov/research-enablement/acorn/commit/ead899d420fc811adcae6b0d8fb5cd7e8279a893))
- Validate lists and more - ([129954d](https://code.ornl.gov/research-enablement/acorn/commit/129954d8fe1f38be0a5e677b76fee878e76b6e87))
- Abstract fuzzy matching data into consts; refine code and logging some more - ([7678ead](https://code.ornl.gov/research-enablement/acorn/commit/7678ead1ef7689ceb68df5081694043f11246de5))
- Add output arg to download subcommand - ([6084aa0](https://code.ornl.gov/research-enablement/acorn/commit/6084aa0cc56dde9298d16eb497070a6c69145d61))
- Rename check to lint - ([0255988](https://code.ornl.gov/research-enablement/acorn/commit/0255988794587f88d23ac1ea4fafefbdfa7abb98))
- Improve acorn check error logging - ([5930cd8](https://code.ornl.gov/research-enablement/acorn/commit/5930cd860e07dc3ab0ed83efb943d4f09eb820bf))
- Refine fuzzy matching and logging - ([88aa081](https://code.ornl.gov/research-enablement/acorn/commit/88aa081bd9e507e34458bf47151ca489a70f5f07))
- Add fuzzy matching for technologies - ([f0aba28](https://code.ornl.gov/research-enablement/acorn/commit/f0aba28d7d8c77032d7800d94f77d1d857e77bff))
- Remove program type and add test data - ([69c2996](https://code.ornl.gov/research-enablement/acorn/commit/69c2996d3c7b5049402c9c168d448ea1e63672ec))
- Separate full validate function - ([70caaf7](https://code.ornl.gov/research-enablement/acorn/commit/70caaf72a16dee47f7b5b43451a436ab3bd4ab63))
- Clean up clippy issues; fix download subcommand - ([3d2599a](https://code.ornl.gov/research-enablement/acorn/commit/3d2599a0825547583e1f83b026fbc5dcd5c1aff0))
- Add validation; specify length rules - ([c707ea6](https://code.ornl.gov/research-enablement/acorn/commit/c707ea6b60b469217632d3f0ce6b32ebaccfd988))
- Consolidate schema files and add program and organization struct definitions - ([5052bcf](https://code.ornl.gov/research-enablement/acorn/commit/5052bcf790cd20c7bcb94ece8ef63d8f9845a6bc))
- Initial project schema capability with check subcommand - ([4ad3cb0](https://code.ornl.gov/research-enablement/acorn/commit/4ad3cb0e6382b09a55038a6918287dc5a0fdafad))
- Add ability to download fact sheets from remote bucket - ([90d7a5a](https://code.ornl.gov/research-enablement/acorn/commit/90d7a5ab465853f1cf5f3a5cea178786d9992d29))
- Add download subcommand dependencies and placeholder code - ([0014a2d](https://code.ornl.gov/research-enablement/acorn/commit/0014a2d0987299e015fa7fa5792d2a9c71a7dea1))
- Initial commit that creates CLI app structure - ([d436e1a](https://code.ornl.gov/research-enablement/acorn/commit/d436e1ae5dab732b465107a036270cd88d92d99b))

### 🐛 Bug Fixes

- *(docs)* Update lint docs to reflect new lint name - ([8392653](https://code.ornl.gov/research-enablement/acorn/commit/8392653d80cd4af36b92226c944145c6ce1e18dc))
- *(test)* Add archive attribute to valid project JSON file - ([a4e0a28](https://code.ornl.gov/research-enablement/acorn/commit/a4e0a280faa18f115f9ce2a60ffacef35d8d8d3d))
- Update fact-sheets.ornl.gov to research.ornl.gov - ([6493909](https://code.ornl.gov/research-enablement/acorn/commit/6493909f18fbc5dfffc54d252bd3251e64d9c8ef))
- Remove un-used import; run make lint - ([fadc0d7](https://code.ornl.gov/research-enablement/acorn/commit/fadc0d753207d72bbccfa90d5dd49e2e442b857c))
- Update technology data after testing format with NSSD bucket - ([54325ba](https://code.ornl.gov/research-enablement/acorn/commit/54325bac7448f7485f99bba52fca0694be0f6e85))
- Reduce font-size to fit all content within PDF - ([1063d3f](https://code.ornl.gov/research-enablement/acorn/commit/1063d3f941566fbe7afa488a82a2d4423cbaa37a))
- Update test files - ([c838a2d](https://code.ornl.gov/research-enablement/acorn/commit/c838a2d51ff73fcf4a1ae830dce8f90988fb23e7))
- Use correct labels in validation errors - ([f86e2e9](https://code.ornl.gov/research-enablement/acorn/commit/f86e2e9013e617e0e4784bbda4c177f79adb5e36))
- Rename methods to technical in highlight test data; Add output parameter to analyze function - ([7e9ac92](https://code.ornl.gov/research-enablement/acorn/commit/7e9ac920d02647f158557a78c910b1ae0a7bfcb5))
- Make lint issue logging more clear - ([623d92b](https://code.ornl.gov/research-enablement/acorn/commit/623d92b2e2a5bed09f43787351dec40cf5d46d89))
- Update org; remove keyword - ([e9fc0f6](https://code.ornl.gov/research-enablement/acorn/commit/e9fc0f6fe9420a56131959f1bd3fbef5158f187d))
- Add run artifacts to gitignore - ([35e54cd](https://code.ornl.gov/research-enablement/acorn/commit/35e54cd5fea9fa36a3eeb5812dc2a1a27655bb48))
- Remove unimplementeds from extract_prose - ([0fadfab](https://code.ornl.gov/research-enablement/acorn/commit/0fadfabceeeeda77803b42f57090abc9f8cf3dd2))
- Add more accept words to clear issues - ([cd0da8a](https://code.ornl.gov/research-enablement/acorn/commit/cd0da8a819ad1d311ad714587af527eef78f10c5))
- Remove test code - ([94ad980](https://code.ornl.gov/research-enablement/acorn/commit/94ad9802aa24921fa7c134de34b5ca38f1afdf4a))
- Update CLI help export examples - ([664a82d](https://code.ornl.gov/research-enablement/acorn/commit/664a82da66f86cf7665cd0390436475839a0a06e))
- Clear issues introduced by merge - ([f7304cb](https://code.ornl.gov/research-enablement/acorn/commit/f7304cb4d565a69961b0cc0cf56ea88b303c40fe))
- Restore introduction char count; tweak invalid test cases - ([f3ce90b](https://code.ornl.gov/research-enablement/acorn/commit/f3ce90b60315f4cb6f51c5d90d28fe4177ead390))

### 📚 Documentation

- Update README - ([548866b](https://code.ornl.gov/research-enablement/acorn/commit/548866b72ad3899bff9bac633d10dada9b2b7b3c))
- Add placeholder files for project documentation elements - ([71cd23e](https://code.ornl.gov/research-enablement/acorn/commit/71cd23ef125214bfebab1ebfe25114ba2e54af0c))
- Update help for download subcommand - ([82c4807](https://code.ornl.gov/research-enablement/acorn/commit/82c4807e38c634cbd0e1c39284ea31854f91d3a1))
- Move help text down one line relative to acorn - ([a7ad9ea](https://code.ornl.gov/research-enablement/acorn/commit/a7ad9eafe5a8378e4389d47a88bac255737bc035))
- Add acorn text art to help message - ([13b6796](https://code.ornl.gov/research-enablement/acorn/commit/13b67964550f4a810836b42a78a6ebdd522ac0e4))

### 🧪 Testing

- Add unit test for Organization asseet parse and load - ([8d2bc1b](https://code.ornl.gov/research-enablement/acorn/commit/8d2bc1b596a88fee296bbcd9103124f7073220d6))
- Add test for get_checksum - ([f636a16](https://code.ornl.gov/research-enablement/acorn/commit/f636a16e89c29adb6fc875a702aa725736929343))
- Move organization test data into folder structure - ([abd15d0](https://code.ornl.gov/research-enablement/acorn/commit/abd15d0d80323dab4ea35ea2ae408f4513056cb9))

### ⚙️ Miscellaneous Tasks

- Add CI configuration - ([c85aa68](https://code.ornl.gov/research-enablement/acorn/commit/c85aa687775c202009c10c55f154ffb2f63df781))

### 🚧 In Progress

- Add highlight footer and caption - ([f3527ee](https://code.ornl.gov/research-enablement/acorn/commit/f3527eef65ee30766a2ac219516eb818342f4e93))
- Even more styles for highlights - ([375df58](https://code.ornl.gov/research-enablement/acorn/commit/375df58dd42d88c7484429295840ac1db7ff8c3a))
- More styles for highlights - ([9eb23b3](https://code.ornl.gov/research-enablement/acorn/commit/9eb23b375284a348abfc5b97efd26b64580e3fe3))
- Pass CLI options through export command; configure browser to change dimensions - ([066e20e](https://code.ornl.gov/research-enablement/acorn/commit/066e20e580121e04d6856f1b75579dde5156eba5))
- Initial capability to resolve contact division - ([c17e626](https://code.ornl.gov/research-enablement/acorn/commit/c17e626997700c8263cd21855115512fe4ca1324))
- Initial addition of organization template - ([7fa6dda](https://code.ornl.gov/research-enablement/acorn/commit/7fa6dda27a3232ad4a66ca4062ed4faf2baf1a3b))
- Initial creation of doctor sub command - ([37dcf79](https://code.ornl.gov/research-enablement/acorn/commit/37dcf7986a3546c2f1b64e4b545fa5c10290b9b4))
- More code cleanup; start get_all_files function to clean up check command code - ([1fc12e6](https://code.ornl.gov/research-enablement/acorn/commit/1fc12e6aab58c744fa0a65f464ba8b5418c61ad6))
- Trying to count schema validation errors - ([10a3547](https://code.ornl.gov/research-enablement/acorn/commit/10a35471140b04edc8ee2ad100979fffe92a1d18))
- Addressed multiple issues and implemented initial vale functional parity - ([4a7b025](https://code.ornl.gov/research-enablement/acorn/commit/4a7b0251bdc94bf8fc0a0455be8337373c4b1b67))
- Initial support for analyzing files with vale - ([5746e7c](https://code.ornl.gov/research-enablement/acorn/commit/5746e7c66de5b2ad60d974f6eb3dd8eec8f973c0))
- Refactor vale code; start config and cmd runner - ([06d832c](https://code.ornl.gov/research-enablement/acorn/commit/06d832c32fa9cef4c0d6ff15bd3136dfba2f88a6))
- Initial capability to download vale binary - ([f38b04d](https://code.ornl.gov/research-enablement/acorn/commit/f38b04d7ecd6ed2fffaaf3ca28c90cb1512ea3db))
- Working on downloading and installing vale - ([edc049a](https://code.ornl.gov/research-enablement/acorn/commit/edc049a57323628cbe54261bb88722c2819f4b55))
- Refine graphic resolution logging - ([6ac2cbb](https://code.ornl.gov/research-enablement/acorn/commit/6ac2cbb09c28cd451fc84219de4eb6b902196b76))
- More progress on graphic href resolution - ([debb7f4](https://code.ornl.gov/research-enablement/acorn/commit/debb7f42dd45ec316695fc9f0353e1a468e26a91))
- Working on format command to resolve first available image in RA folder for PDF export - ([02469bd](https://code.ornl.gov/research-enablement/acorn/commit/02469bd95ed9e56d4edb0f8bd73545043afbfe98))
- Add folder support for factsheet export target; abstract pdf generation into function; prepare for graphic href resolution - ([17db378](https://code.ornl.gov/research-enablement/acorn/commit/17db37839a1098ea6bbc82177a830ea3fbc197b5))
- Adding PDF image via relative path - ([2714900](https://code.ornl.gov/research-enablement/acorn/commit/2714900904334033d37f7f02f5b01bace7e31c14))
- Adding pagination support - ([971c85b](https://code.ornl.gov/research-enablement/acorn/commit/971c85b906356885e60c8b4c39ed508cdd1d21c5))
- Initial addition of Endpoint data model - ([db5433e](https://code.ornl.gov/research-enablement/acorn/commit/db5433edca9e155486c87a7b41356fe545f99672))
- Add comment for validating highlight approach - ([d807316](https://code.ornl.gov/research-enablement/acorn/commit/d80731613688da5b84f71e566d691c18a00c0bd2))
- Implementing fuzzy matching when ingesting fact sheet data - ([79a4d15](https://code.ornl.gov/research-enablement/acorn/commit/79a4d155c2613d1d8fc800930bda7a5de5b5f919))
- Add fuzzy matching library and example test - ([72c13b0](https://code.ornl.gov/research-enablement/acorn/commit/72c13b08064da0c57c18f3ce9933d4ed43f40bf7))

### 🤓 Developer Experience

- Add vscode settings - ([8eea7dc](https://code.ornl.gov/research-enablement/acorn/commit/8eea7dc8064acb6b50664efee8c63ace24c2102a))
