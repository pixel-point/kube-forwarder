<template>
  <portal to="modal">
    <div class="modal">
      <div class="modal__overlay" />
      <div class="modal__window-container">
        <div class="modal__window">
          <slot />
        </div>
      </div>
    </div>
  </portal>
</template>

<style lang="scss">
.modal {
  position: fixed;
  top: 0;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  justify-content: center;
  flex-direction: column;
}

.modal__window-container {
  width: 100%;
  overflow: auto;
  padding: 20px 20px 0;
}

.modal__window {
  background: #fff;
  z-index: 10;
  position: relative;
  overflow: hidden;
  min-width: 520px;
  border-radius: 10px;
  margin: 0 auto 20px;
}

.modal__overlay {
  position: fixed;
  top: 0;
  bottom: 0;
  left: 0;
  right: 0;
  background-color: rgba(13, 19, 37, 0.89);
}

.modal__content {
  margin: 20px
}

.modal__actions {
  margin: 20px;
  display: flex;
}
</style>
