let analytics

if (process.env.NODE_ENV === 'production') {
  const Analytics = require('electron-ga')
  analytics = new Analytics(process.env.GA_TRACKING_ID)
} else {
  analytics = {
    send: () => {}
  }
}

export default analytics
