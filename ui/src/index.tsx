/* @refresh reload */
import { render } from 'solid-js/web'

import "./global"
import './index.css'
import App from './App'
// import { ConfigProvider } from './context/config'

const root = document.body

render(() =>
    <App />
    , root!)
