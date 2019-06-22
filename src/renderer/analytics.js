import Analytics from 'electron-ga'

export default new Analytics(process.env.GA_TRACKING_ID)
