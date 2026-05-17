const fs = require('fs')
const path = require('path')

const indexPath = path.join(__dirname, '..', 'index.js')
let content = fs.readFileSync(indexPath, 'utf8')

content = content.replace(
  'let nativeBinding = null\n',
  "let nativeBinding = null\nconst _expectedVersion = require('./package.json').version\n",
)

content = content.replace(/bindingPackageVersion !== '[^']+'/g, 'bindingPackageVersion !== _expectedVersion')

content = content.replace(
  /expected [0-9]+\.[0-9]+\.[0-9]+ but got/g,
  'expected ${_expectedVersion} but got',
)

fs.writeFileSync(indexPath, content)
