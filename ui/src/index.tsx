/* @refresh reload */
import { render } from 'solid-js/web'

import './index.css'
import App from './App'
// import { ConfigProvider } from './context/config'

const root = document.getElementById('root')

render(() =>
    <App />
    , root!)
