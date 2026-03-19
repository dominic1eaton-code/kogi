import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletBenefitsComponent } from './wallet-benefits.component';

describe('WalletBenefitsComponent', () => {
  let component: WalletBenefitsComponent;
  let fixture: ComponentFixture<WalletBenefitsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletBenefitsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletBenefitsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
