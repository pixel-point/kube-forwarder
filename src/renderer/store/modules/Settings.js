import Vue from 'vue'
import { remote } from 'electron'

const { systemPreferences } = remote

const defaultState = {
  theme: systemPreferences.isDarkMode() ? 'dark' : 'light'
}

const mutations = {
  SET(state, { key, value }) {
    Vue.set(state, key, value)
  }
}

const actions = {
  setSetting({ commit }, payload) {
    commit('SET', payload)
  }
}

export default {
  namespaced: true,
  state: defaultState,
  mutations,
  actions
}
