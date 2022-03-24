// see types of prompts:
// https://github.com/enquirer/enquirer/tree/master/examples
//
module.exports = [
  {
    type: 'input',
    name: 'action',
    message: "What's the packet action?"
  },
  {
    type: 'input',
    name: 'family',
    message: "What's the packet family?"
  }
]
