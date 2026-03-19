import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletCampaignsComponent } from './wallet-campaigns.component';

describe('WalletCampaignsComponent', () => {
  let component: WalletCampaignsComponent;
  let fixture: ComponentFixture<WalletCampaignsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletCampaignsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletCampaignsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
