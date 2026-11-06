/* eslint-disable no-template-curly-in-string */
import { config as dotenv } from 'dotenv'
import type { Configuration } from 'electron-builder'
import { resolve } from 'path'

dotenv()

const linuxPackageScripts = {
  afterInstall: 'build/linux/after-install.tpl',
  afterRemove: 'build/linux/after-remove.tpl',
  appArmorProfile: 'build/linux/apparmor-profile.tpl',
  fpm: [`--after-upgrade=${resolve(__dirname, 'linux', 'after-install.tpl')}`],
}

export const config = {
  // zenith: Zenith Launcher identity (fork of XMCL, see docs/ZENITH-LAUNCHER.md)
  productName: 'Zenith Launcher',
  appId: 'dev.zenith.launcher',
  copyright: 'Copyright (c) Zenith contributors. Based on XMCL, Copyright (c) 2019 Voxelum (MIT)',
  directories: {
    output: 'build/output',
    buildResources: 'build',
    app: '.',
  },
  protocols: {
    name: 'Zenith Launcher',
    schemes: ['zenith-launcher'],
  },
  // assign publish for auto-updater
  // set this to your own repo!
  publish: [{
    provider: 'github',
    owner: 'gitzetazero',
    repo: 'zenith',
  }],
  files: [{
    from: 'dist',
    to: '.',
    filter: ['**/*.js', '**/*.ico', '**/*.png', '**/*.webp', '**/*.svg', '*.node', '*.dll', '**/*.html', '**/*.css', '**/*.woff2', '**/*.wasm'],
  }, {
    from: '.',
    to: '.',
    filter: 'package.json',
  }],
  extraResources: [{
    // zenith: the verified Zenith bridge, downloaded by the release workflow (tools/zenith/fetch-bridge.mjs)
    from: 'build/zenith',
    to: 'zenith',
    filter: ['zenith-bridge-windows-x64.exe'],
  }],
  artifactName: 'Zenith-Launcher-${platform}-${arch}.${ext}',
  appx: {
    displayName: 'Zenith Launcher',
    applicationId: 'zenith-launcher',
    identityName: 'zenith-launcher',
    backgroundColor: 'transparent',
    publisher: process.env.PUBLISHER,
    publisherDisplayName: 'CI010',
    setBuildNumber: true,
  },
  dmg: {
    artifactName: 'zenith-launcher-${version}-${arch}.${ext}',
    contents: [
      {
        x: 410,
        y: 150,
        type: 'link',
        path: '/Applications',
      },
      {
        x: 130,
        y: 150,
        type: 'file',
      },
    ],
  },
  mac: {
    icon: 'icons/dark.icns',
    darkModeSupport: true,
    target: [
      {
        target: 'dmg',
        arch: ['arm64', 'x64'],
      },
    ],
    extendInfo: {
      NSLocalNetworkUsageDescription: 'XMCL and Minecraft need access to your local network to discover and connect to LAN games and servers.',
      NSMicrophoneUsageDescription: 'A Minecraft mod wants to access your microphone.',
      NSCameraUsageDescription: 'Please give us access to your camera',
      'com.apple.security.device.audio-input': true,
      'com.apple.security.device.camera': true,
    },
  },
  win: {
    signtoolOptions: {
      certificateFile: undefined as string | undefined,
      publisherName: 'Zenith Launcher', // zenith: unsigned builds; XMCL's publisher is not used
    },
    icon: 'icons/dark.ico',
    electronLanguages: ['en-US'],
    // zenith: NSIS installer (per user, no admin) + portable exe, x64 only
    target: [
      { target: 'nsis', arch: ['x64'] },
      { target: 'portable', arch: ['x64'] },
    ],
  },
  nsis: {
    oneClick: false,
    perMachine: false,
    allowElevation: false,
    allowToChangeInstallationDirectory: true,
    shortcutName: 'Zenith Launcher',
    artifactName: 'Zenith-Launcher-Setup-x64.${ext}',
  },
  portable: {
    artifactName: 'Zenith-Launcher-Portable-x64.${ext}',
  },
  linux: {
    executableName: 'xmcl',
    electronLanguages: ['en-US'],
    desktop: {
      entry: {
        MimeType: 'x-scheme-handler/xmcl',
        StartupWMClass: 'xmcl',
      },
    },
    category: 'Game',
    icon: 'icons/dark.icns',
    artifactName: 'zenith-launcher-${version}-${arch}.${ext}',
    target: [
      { target: 'deb', arch: ['x64', 'arm64'] },
      { target: 'rpm', arch: ['x64', 'arm64'] },
      { target: 'AppImage', arch: ['x64', 'arm64'] },
      { target: 'tar.xz', arch: ['x64', 'arm64'] },
      { target: 'pacman', arch: ['x64', 'arm64'] },
    ],
  },
  deb: linuxPackageScripts,
  rpm: linuxPackageScripts,
  pacman: linuxPackageScripts,
  snap: {
    publish: [
      'github',
    ],
  },
} satisfies Configuration
